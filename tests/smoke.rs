//! A bunch of ways to use async/await syntax.
//!
//! This is mostly a test for this repository itself, not necessarily serving
//! much more purpose than that.

#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;

use std::io;

use futures::prelude::*;

#[async]
fn foo() -> impl Future<Item=i32, Error=i32> {
    Ok(1)
}

#[async]
extern fn _foo1() -> impl Future<Item=i32, Error=i32> {
    Ok(1)
}

#[async]
unsafe fn _foo2() -> impl Future<Item=i32, Error=io::Error> {
    Ok(1)
}

#[async]
unsafe extern fn _foo3() -> impl Future<Item=i32, Error=io::Error> {
    Ok(1)
}

#[async]
pub fn _foo4() -> impl Future<Item=i32, Error=io::Error> {
    Ok(1)
}

#[async]
fn _foo5<T: Clone + 'static>(t: T) -> impl Future<Item=T, Error=i32> {
    Ok(t.clone())
}

#[async]
fn _foo6(ref a: i32) -> impl Future<Item=i32, Error=i32> {
    Err(*a)
}

#[async]
fn _foo7<T>(t: T) -> impl Future<Item=T, Error=i32>
    where T: Clone + 'static,
{
    Ok(t.clone())
}

#[async]
fn _foo8(a: i32, b: i32) -> Box<Future<Item=i32, Error=i32>> {
    return Ok(a + b)
}

#[async]
fn _bar() -> impl Future<Item=i32, Error=i32> {
    await!(foo())
}

#[async]
fn _bar2() -> impl Future<Item=i32, Error=i32> {
    let a = await!(foo())?;
    let b = await!(foo())?;
    Ok(a + b)
}

#[async]
fn _bar3() -> impl Future<Item=i32, Error=i32> {
    let (a, b) = await!(foo().join(foo()))?;
    Ok(a + b)
}

#[async]
fn _bar4() -> impl Future<Item=i32, Error=i32> {
    let mut cnt = 0;
    #[async]
    for x in futures::stream::iter_ok::<_, i32>(vec![1, 2, 3, 4]) {
        cnt += x;
    }
    Ok(cnt)
}

#[async]
fn _stream1() -> impl Stream<Item=i32, Error=i32> {
    stream_yield!(0);
    stream_yield!(1);
    Ok(())
}

#[async]
fn _stream2<T: Clone + 'static>(t: T) -> impl Stream<Item=T, Error=i32> {
    stream_yield!(t.clone());
    stream_yield!(t.clone());
    Ok(())
}

#[async]
fn _stream3() -> impl Stream<Item=i32, Error=i32> {
    let mut cnt = 0;
    #[async]
    for x in futures::stream::iter_ok::<_, i32>(vec![1, 2, 3, 4]) {
        cnt += x;
        stream_yield!(x);
    }
    Err(cnt)
}

#[async]
fn _stream4() -> Box<Stream<Item=u64, Error=i32>> {
    stream_yield!(0);
    stream_yield!(1);
    Ok(())
}

mod foo { pub struct Foo(pub i32); }

#[async]
pub fn stream5() -> Box<Stream<Item=foo::Foo, Error=i32>> {
    stream_yield!(foo::Foo(0));
    stream_yield!(foo::Foo(1));
    Ok(())
}

#[async]
pub fn _stream6() -> Box<Stream<Item=i32, Error=i32>> {
    #[async]
    for foo::Foo(i) in stream5() {
        stream_yield!(i * i);
    }
    Ok(())
}

#[async]
pub fn _stream7() -> impl Stream<Item=(), Error=i32> {
    stream_yield!(());
    Ok(())
}

#[async]
pub fn _stream8() -> impl Stream<Item=[u32; 4], Error=i32> {
    stream_yield!([1, 2, 3, 4]);
    Ok(())
}

// struct A(i32);
//
// impl A {
//     #[async]
//     fn a_foo(self) -> impl Future<Item=i32, Error=i32> {
//         Ok(self.0)
//     }
//
//     #[async]
//     fn _a_foo2(self: Box<Self>) -> impl Future<Item=i32, Error=i32> {
//         Ok(self.0)
//     }
// }

// trait B {
//     #[async]
//     fn b(self) -> impl Future<Item=i32, Error=i32>;
// }
//
// impl B for A {
//     #[async]
//     fn b(self) -> impl Future<Item=i32, Error=i32> {
//         Ok(self.0)
//     }
// }

#[test]
fn main() {
    assert_eq!(foo().wait(), Ok(1));
    assert_eq!(_bar().wait(), Ok(1));
    assert_eq!(_bar2().wait(), Ok(2));
    assert_eq!(_bar3().wait(), Ok(2));
    assert_eq!(_bar4().wait(), Ok(10));
    assert_eq!(_foo6(8).wait(), Err(8));
    // assert_eq!(A(11).a_foo().wait(), Ok(11));
    assert_eq!(loop_in_loop().wait(), Ok(true));
}

#[async]
fn loop_in_loop() -> impl Future<Item=bool, Error=i32> {
    let mut cnt = 0;
    let vec = vec![1, 2, 3, 4];
    #[async]
    for x in futures::stream::iter_ok::<_, i32>(vec.clone()) {
        #[async]
        for y in futures::stream::iter_ok::<_, i32>(vec.clone()) {
            cnt += x * y;
        }
    }

    let sum = (1..5).map(|x| (1..5).map(|y| x * y).sum::<i32>()).sum::<i32>();
    Ok(cnt == sum)
}

#[async]
fn poll_stream_after_error_stream() -> impl Stream<Item=i32, Error=()> {
    stream_yield!(5);
    Err(())
}

#[test]
fn poll_stream_after_error() {
    let mut s = poll_stream_after_error_stream();
    assert_eq!(s.poll(), Ok(Async::Ready(Some(5))));
    assert_eq!(s.poll(), Err(()));
    assert_eq!(s.poll(), Ok(Async::Ready(None)));
}
