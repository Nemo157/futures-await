#![allow(warnings)]
#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;

use futures::prelude::*;

fn bar<'a>(a: &'a str) -> Box<Future<Item = i32, Error = u32> + 'a> {
    panic!()
}

#[async]
fn foo(a: String) -> impl Future<Item=i32, Error=u32> {
    await!(bar(&a))?;
    drop(a);
    Ok(1)
}

#[async]
fn foos(a: String) -> impl Stream<Item=i32, Error=u32> {
    await!(bar(&a))?;
    drop(a);
    stream_yield!(5);
    Ok(())
}

fn main() {}
