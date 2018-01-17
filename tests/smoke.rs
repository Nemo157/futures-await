//! A bunch of ways to use async/await syntax.
//!
//! This is mostly a test f r this repository itself, not necessarily serving
//! much more purpose than that.

#![feature(proc_macro, conservative_impl_trait, generators, pin)]

extern crate futures_await as futures;

use futures::executor::{self, Executor};

use std::io;

use futures::Never;
use futures::future::poll_fn;
use futures::stable::{StableFuture, block_on_stable};
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
    where T: Clone,
{
    Ok(t.clone())
}

#[async]
fn _foo8(a: i32, b: i32) -> Box<Future<Item=i32, Error=i32>> {
    return Ok(a + b)
}

#[async]
fn _foo9() -> Box<Future<Item=(), Error=Never> + Send> {
    Ok(())
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
fn _stream1() -> impl Stream<Item=u64, Error=i32> {
    stream_yield!(0);
    stream_yield!(1);
    Ok(())
}

#[async]
fn _stream2<T: Clone>(t: T) -> impl Stream<Item=T, Error=i32> {
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

struct A(i32);

impl A {
    #[async]
    fn a_foo(self) -> impl StableFuture<Item=i32, Error=i32> {
        Ok(self.0)
    }

    #[async]
    fn _a_foo2(self: Box<Self>) -> impl Future<Item=i32, Error=i32> {
        Ok(self.0)
    }
}

#[async]
fn await_item_stream() -> impl Stream<Item=u64, Error=i32> {
    stream_yield!(0);
    stream_yield!(1);
    Ok(())
}

#[async]
fn test_await_item() -> impl Future<Item=(), Error=Never> {
    let mut stream = await_item_stream();

    assert_eq!(await_item!(stream), Ok(Some(0)));
    assert_eq!(await_item!(stream), Ok(Some(1)));
    assert_eq!(await_item!(stream), Ok(None));

    Ok(())
}

#[test]
fn main() {
    assert_eq!(executor::block_on(foo()), Ok(1));
    assert_eq!(executor::block_on(foo()), Ok(1));
    assert_eq!(executor::block_on(_bar()), Ok(1));
    assert_eq!(executor::block_on(_bar2()), Ok(2));
    assert_eq!(executor::block_on(_bar3()), Ok(2));
    assert_eq!(executor::block_on(_bar4()), Ok(10));
    assert_eq!(executor::block_on(_foo6(8)), Err(8));
    assert_eq!(block_on_stable(A(11).a_foo()), Ok(11));
    assert_eq!(executor::block_on(loop_in_loop()), Ok(true));
    assert_eq!(executor::block_on(test_await_item()), Ok(()));
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
    assert_eq!(executor::block_on(poll_fn(|ctx| s.poll_next(ctx))), Ok(Some(5)));
    assert_eq!(executor::block_on(poll_fn(|ctx| s.poll_next(ctx))), Err(()));
    assert_eq!(executor::block_on(poll_fn(|ctx| s.poll_next(ctx))), Ok(None));
}

#[test]
fn run_boxed_future_in_cpu_pool() {
    let mut pool = executor::ThreadPool::new();
    pool.spawn(_foo9()).unwrap();
}
