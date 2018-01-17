#![feature(proc_macro, conservative_impl_trait, generators, pin)]

extern crate futures_await as futures;

use futures::prelude::*;

#[async]
fn foo() -> u32 {
    3
}

#[async]
fn bar() -> Box<u32> {
    3
}

fn main() {}
