//! Procedural macro for the `#[async]` attribute.
//!
//! This crate is an implementation of the `#[async]` attribute as a procedural
//! macro. This is nightly-only for now as it's using the unstable features of
//! procedural macros. Furthermore it's generating code that's using a new
//! keyword, `yield`, and a new construct, generators, both of which are also
//! unstable.
//!
//! Currently this crate depends on `syn` and `quote` to do all the heavy
//! lifting, this is just a very small shim around creating a closure/future out
//! of a generator.

#![feature(proc_macro, match_default_bindings)]
#![recursion_limit = "128"]

extern crate proc_macro2;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::Span;
use proc_macro::{TokenStream, TokenTree, Delimiter, TokenNode};
use quote::{Tokens, ToTokens};
use syn::*;
use syn::fold::Fold;

macro_rules! quote_cs {
    ($($t:tt)*) => (quote_spanned!(Span::call_site() => $($t)*))
}

#[proc_macro_attribute]
pub fn async(attribute: TokenStream, function: TokenStream) -> TokenStream {
    // Handle arguments to the #[async] attribute, if any
    let attribute = attribute.to_string();
    if attribute != "" {
        panic!("the #[async] attribute takes no arguments");
    };

    // Parse our item, expecting a function. This function may be an actual
    // top-level function or it could be a method (typically dictated by the
    // arguments). We then extract everything we'd like to use.
    let ItemFn {
        ident,
        vis,
        unsafety,
        constness,
        abi,
        block,
        decl,
        attrs,
        ..
    } = match syn::parse(function).expect("failed to parse tokens as a function") {
        Item::Fn(item) => item,
        _ => panic!("#[async] can only be applied to functions"),
    };
    let FnDecl {
        inputs,
        output,
        variadic,
        generics,
        fn_token,
        ..
    } = { *decl };
    let where_clause = &generics.where_clause;
    assert!(variadic.is_none(), "variadic functions cannot be async");
    let (output, rarrow_token) = match output {
        ReturnType::Type(rarrow_token, t) => (*t, rarrow_token),
        ReturnType::Default => {
            (TypeTuple {
                elems: Default::default(),
                paren_token: Default::default(),
            }.into(), Default::default())
        }
    };

    // We've got to get a bit creative with our handling of arguments. For a
    // number of reasons we translate this:
    //
    //      fn foo(ref a: u32) -> Result<u32, u32> {
    //          // ...
    //      }
    //
    // into roughly:
    //
    //      fn foo(__arg_0: u32) -> impl Future<...> {
    //          gen_move(move || {
    //              let ref a = __arg0;
    //
    //              // ...
    //          })
    //      }
    //
    // The intention here is to ensure that all local function variables get
    // moved into the generator we're creating, and they're also all then bound
    // appropriately according to their patterns and whatnot.
    //
    // We notably skip everything related to `self` which typically doesn't have
    // many patterns with it and just gets captured naturally.
    let mut inputs_no_patterns = Vec::new();
    let mut patterns = Vec::new();
    let mut temp_bindings = Vec::new();
    for (i, input) in inputs.into_iter().enumerate() {
        // `self: Box<Self>` will get captured naturally
        let mut is_input_no_pattern = false;
        if let FnArg::Captured(ref arg) = input {
            if let Pat::Ident(PatIdent { ref ident, ..}) = arg.pat {
                if ident == "self" {
                    is_input_no_pattern = true;
                }
            }
        }
        if is_input_no_pattern {
            inputs_no_patterns.push(input);
            continue
        }

        match input {
            FnArg::Captured(ArgCaptured {
                pat: syn::Pat::Ident(syn::PatIdent {
                    by_ref: None,
                    ..
                }),
                ..
            }) => {
                inputs_no_patterns.push(input);
            }

            // `ref a: B` (or some similar pattern)
            FnArg::Captured(ArgCaptured { pat, ty, colon_token }) => {
                patterns.push(pat);
                let ident = Ident::from(format!("__arg_{}", i));
                temp_bindings.push(ident.clone());
                let pat = PatIdent {
                    by_ref: None,
                    mutability: None,
                    ident: ident,
                    subpat: None,
                };
                inputs_no_patterns.push(ArgCaptured {
                    pat: pat.into(),
                    ty,
                    colon_token,
                }.into());
            }

            // Other `self`-related arguments get captured naturally
            _ => {
                inputs_no_patterns.push(input);
            }
        }
    }


    // This is the point where we handle
    //
    //      #[async]
    //      for x in y {
    //      }
    //
    // Basically just take all those expression and expand them.
    let block = ExpandAsyncFor.fold_block(*block);

    let block_inner = quote_cs! {
        #( let #patterns = #temp_bindings; )*
        #block
    };
    let mut result = Tokens::new();
    block.brace_token.surround(&mut result, |tokens| {
        block_inner.to_tokens(tokens);
    });
    syn::token::Semi([block.brace_token.0]).to_tokens(&mut result);

    let gen_body_inner = quote_cs! {
        let __e = #result

        // Ensure that this closure is a generator, even if it doesn't
        // have any `yield` statements.
        #[allow(unreachable_code)]
        {
            return __e;
            loop { yield ::futures::__rt::Async::Pending }
        }
    };
    let mut gen_body = Tokens::new();
    block.brace_token.surround(&mut gen_body, |tokens| {
        gen_body_inner.to_tokens(tokens);
    });

    // Give the invocation of the `gen` function the same span as the output
    // as currently errors related to it being a result are targeted here. Not
    // sure if more errors will highlight this function call...
    let output_span = first_last(&output);
    let gen_function = quote_cs! { ::futures::__rt::gen_async };
    let gen_function = respan(gen_function.into(), &output_span);
    // TODO: Don't use string matching for this
    let (pinned, boxed) = match output {
        Type::Path(_) => (false, true),
        Type::ImplTrait(TypeImplTrait { ref bounds, .. }) => {
            if let Some(TypeParamBound::Trait(bound)) = bounds.first().map(punctuated::Pair::into_value) {
                if let Some(segment) = bound.path.segments.last().map(punctuated::Pair::into_value) {
                    match segment.ident.as_ref() {
                        "Future" => (false, false),
                        "Stream" => (false, false),
                        "StableFuture" => (true, false),
                        "StableStream" => (true, false),
                        _ => {
                            panic!("#[async] function with an `impl Trait` return type must have one of\
                                `Future`, `Stream`, `StableFuture` or `StableStream` as the first bound");
                        }
                    }
                } else {
                    panic!("#[async] function with an `impl Trait` return type must have one of\
                            `Future`, `Stream`, `StableFuture` or `StableStream` as the first bound");
                }
            } else {
                panic!("#[async] function with an `impl Trait` return type must have one of\
                        `Future`, `Stream`, `StableFuture` or `StableStream` as the first bound");
            }
        }
        _ => {
            panic!("#[async] function return type must be one of `impl \
                    Future`, `impl Stream`, `Box<Future>` or `Box<Stream>`");
        }
    };
    let body_inner = if pinned {
        quote_cs! {
            #gen_function (#[allow(unused_unsafe)] unsafe { static move || #gen_body })
        }
    } else {
        quote_cs! {
            #gen_function (move || #gen_body)
        }
    };
    let body_inner = if boxed {
        let body = quote_cs! { ::futures::__rt::std::boxed::Box::new(#body_inner) };
        respan(body.into(), &output_span)
    } else {
        body_inner.into()
    };
    let mut body = Tokens::new();
    block.brace_token.surround(&mut body, |tokens| {
        body_inner.to_tokens(tokens);
    });

    let transformed = quote_cs! {
        #(#attrs)*
        #vis #unsafety #abi #constness
        #fn_token #ident #generics(#(#inputs_no_patterns),*)
            #rarrow_token #output
            #where_clause
        #body
    };

    // println!("{}", transformed);
    transformed.into()
}

#[proc_macro]
pub fn async_block(input: TokenStream) -> TokenStream {
    let input = TokenStream::from(TokenTree {
        kind: TokenNode::Group(Delimiter::Brace, input),
        span: proc_macro::Span::def_site(),
    });
    let expr = syn::parse(input)
        .expect("failed to parse tokens as an expression");
    let expr = ExpandAsyncFor.fold_expr(expr);

    let mut tokens = quote_cs! {
        ::futures::__rt::gen_move
    };

    // Use some manual token construction here instead of `quote_cs!` to ensure
    // that we get the `call_site` span instead of the default span.
    let span = Span::call_site();
    syn::token::Paren(span).surround(&mut tokens, |tokens| {
        syn::token::Move(span).to_tokens(tokens);
        syn::token::OrOr([span, span]).to_tokens(tokens);
        syn::token::Brace(span).surround(tokens, |tokens| {
            (quote_cs! {
                if false { yield ::futures::__rt::Async::Pending }
            }).to_tokens(tokens);
            expr.to_tokens(tokens);
        });
    });

    tokens.into()
}

struct ExpandAsyncFor;

impl Fold for ExpandAsyncFor {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        let expr = fold::fold_expr(self, expr);
        let mut async = false;
        {
            let attrs = match expr {
                Expr::ForLoop(syn::ExprForLoop { ref attrs, .. }) => attrs,
                _ => return expr,
            };
            if attrs.len() == 1 {
                // TODO: more validation here
                if attrs[0].path.segments.first().unwrap().value().ident == "async" {
                    async = true;
                }
            }
        }
        if !async {
            return expr
        }
        let all = match expr {
            Expr::ForLoop(item) => item,
            _ => panic!("only for expressions can have #[async]"),
        };
        let ExprForLoop { pat, expr, body, label, .. } = all;

        // Basically just expand to a `poll` loop
        let tokens = quote_cs! {{
            let mut __stream = #expr;
            #label
            loop {
                let #pat = {
                    let r = {
                        let pin = unsafe {
                            ::futures::__rt::pin_api::mem::Pin::new_unchecked(&mut __stream)
                        };
                        ::futures::__rt::in_ctx(|ctx| ::futures::__rt::StableStream::poll_next(pin, ctx))
                    };
                    match r? {
                        ::futures::__rt::Async::Ready(e) => {
                            match e {
                                ::futures::__rt::std::option::Option::Some(e) => e,
                                ::futures::__rt::std::option::Option::None => break,
                            }
                        }
                        ::futures::__rt::Async::Pending => {
                            yield ::futures::__rt::Async::Pending;
                            continue
                        }
                    }
                };

                #body
            }
        }};
        syn::parse(tokens.into()).unwrap()
    }

    // Don't recurse into items
    fn fold_item(&mut self, item: Item) -> Item {
        item
    }
}

fn first_last(tokens: &ToTokens) -> (Span, Span) {
    let mut spans = Tokens::new();
    tokens.to_tokens(&mut spans);
    let good_tokens = proc_macro2::TokenStream::from(spans).into_iter().collect::<Vec<_>>();
    let first_span = good_tokens.first().map(|t| t.span).unwrap_or(Span::def_site());
    let last_span = good_tokens.last().map(|t| t.span).unwrap_or(first_span);
    (first_span, last_span)
}

fn respan(input: proc_macro2::TokenStream,
          &(first_span, last_span): &(Span, Span)) -> proc_macro2::TokenStream {
    let mut new_tokens = input.into_iter().collect::<Vec<_>>();
    if let Some(token) = new_tokens.first_mut() {
        token.span = first_span;
    }
    for token in new_tokens.iter_mut().skip(1) {
        token.span = last_span;
    }
    new_tokens.into_iter().collect()
}
