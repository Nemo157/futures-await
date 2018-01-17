#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;

use futures::prelude::*;

#[async]
fn foo() -> impl Future<Item=A, Error=u32> {
    Err(3)
}

#[async]
fn foos() -> impl Stream<Item=A, Error=u32> {
    Err(3)
}

fn main() {}
