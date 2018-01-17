#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;

use futures::prelude::*;

#[async]
fn foo<T>(t: T) -> impl Future<Item=T, Error=u32> {
    Ok(t)
}

#[async]
fn foos<T>(t: T) -> impl Stream<Item=T, Error=u32> {
    stream_yield!(t);
    Ok(())
}

#[async]
fn foos2<T>(t: T) -> impl Stream<Item=i32, Error=u32> {
    Ok(())
}

fn main() {}
