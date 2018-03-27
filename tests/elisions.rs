#![feature(proc_macro, conservative_impl_trait, generators, underscore_lifetimes, pin)]

extern crate futures_await as futures;

use futures::stable::{StableFuture, StableStream, block_on_stable};
use futures::prelude::*;

struct Ref<'a, T: 'a>(&'a T);

#[async]
fn references(x: &i32) -> impl StableFuture<Item=i32, Error=i32> + '_ {
    Ok(*x)
}

#[async]
fn new_types(x: Ref<'_, i32>) -> impl StableFuture<Item=i32, Error=i32> + '_ {
    Ok(*x.0)
}

#[async(unpinned)]
fn references_move(x: &i32) -> impl Future<Item=i32, Error=i32> + '_ {
    Ok(*x)
}

#[async]
fn _streams(x: &i32) -> impl StableStream<Item=i32, Error=i32> + '_ {
    stream_yield!(*x);
    Ok(())
}

struct Foo(i32);

impl Foo {
    #[async]
    fn foo(&self) -> impl StableFuture<Item=&i32, Error=i32> {
        Ok(&self.0)
    }
}

#[async]
fn single_ref(x: &i32) -> impl StableFuture<Item=&i32, Error=i32> {
    Ok(x)
}

#[async]
fn check_for_name_collision<'_async0, T>(_x: &T, _y: &'_async0 i32) -> impl StableFuture<Item=(), Error=()> {
    Ok(())
}

#[test]
fn main() {
    let x = 0;
    let foo = Foo(x);
    assert_eq!(block_on_stable(references(&x)), Ok(x));
    assert_eq!(block_on_stable(new_types(Ref(&x))), Ok(x));
    assert_eq!(block_on_stable(references_move(&x)), Ok(x));
    assert_eq!(block_on_stable(single_ref(&x)), Ok(&x));
    assert_eq!(block_on_stable(foo.foo()), Ok(&x));
    assert_eq!(block_on_stable(check_for_name_collision(&x, &x)), Ok(()));
}
