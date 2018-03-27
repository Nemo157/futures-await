#![feature(proc_macro, conservative_impl_trait, generators, pin)]

extern crate futures_await as futures;

use futures::prelude::*;

#[async(unpinned)]
fn foo() -> impl Future<Item=Left, Error=u32> {
    Err(3)
}

#[async(unpinned)]
fn foos() -> impl Stream<Item=Left, Error=u32> {
    Err(3)
}

fn main() {}
