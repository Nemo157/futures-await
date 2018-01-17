#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;

use futures::prelude::*;

#[async]
fn foo() -> impl Future<Item=Left, Error=u32> {
    Err(3)
}

<<<<<<< HEAD
#[async]
fn foos() -> impl Stream<Item=Left, Error=u32> {
    Err(3)
}

fn main() {}
