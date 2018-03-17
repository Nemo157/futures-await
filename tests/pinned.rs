#![feature(proc_macro, conservative_impl_trait, generators, underscore_lifetimes)]

extern crate futures_await as futures;

use futures::stable::{StableFuture, StableStream, block_on_stable};
use futures::prelude::*;

#[async]
fn foo() -> impl StableFuture<Item=i32, Error=i32> {
    Ok(1)
}

#[async]
fn bar(x: &i32) -> impl StableFuture<Item=i32, Error=i32> + '_ {
    Ok(*x)
}

#[async]
fn baz(x: i32) -> impl StableFuture<Item=i32, Error=i32> {
    await!(bar(&x))
}

#[async]
fn _stream1() -> impl StableStream<Item=u64, Error=i32> {
    fn integer() -> u64 { 1 }
    let x = &integer();
    stream_yield!(0);
    stream_yield!(*x);
    Ok(())
}

#[async]
pub fn uses_async_for() -> impl StableFuture<Item=Vec<u64>, Error=i32> {
    let mut v = vec![];
    #[async]
    for i in _stream1() {
        v.push(i);
    }
    Ok(v)
}

#[test]
fn main() {
    assert_eq!(block_on_stable(foo()), Ok(1));
    assert_eq!(block_on_stable(bar(&1)), Ok(1));
    assert_eq!(block_on_stable(baz(17)), Ok(17));
    assert_eq!(block_on_stable(uses_async_for()), Ok(vec![0, 1]));
}
