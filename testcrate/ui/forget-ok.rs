#![feature(proc_macro, conservative_impl_trait, generators, pin)]

extern crate futures_await as futures;

use futures::prelude::*;

#[async(unpinned)]
fn foo() -> impl Future<Item=(), Error=()> {
}

#[async(unpinned)]
fn foos() -> impl Stream<Item=i32, Error=()> {
}

fn main() {}
