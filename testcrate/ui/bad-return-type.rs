#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;

use futures::prelude::*;

#[async]
fn foobar() -> impl Future<Item=Option<i32>, Error=()> {
    let val = Some(42);
    if val.is_none() {
        return Ok(None)
    }
    let val = val.unwrap();
    Ok(val)
}

#[async]
fn foobars() -> impl Stream<Item=Option<i32>, Error=()> {
    let val = Some(42);
    if val.is_none() {
        stream_yield!(None);
        return Ok(())
    }
    let val = val.unwrap();
    stream_yield!(val);
    Ok(())
}

#[async]
fn tuple() -> impl Future<Item=(i32, i32), Error=()> {
    if false {
        return Ok(3);
    }
    Ok((1, 2))
}

fn main() {}
