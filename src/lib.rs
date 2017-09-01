//! Runtime support for the async/await syntax for futures.
//!
//! This crate serves as a masquerade over the `futures` crate itself,
//! reexporting all of its contents. It's intended that you'll do:
//!
//! ```
//! extern crate futures_await as futures;
//! ```
//!
//! This crate adds a `prelude` module which contains various traits as well as
//! the `async` and `await` macros you'll likely want to use.
//!
//! See the crates's README for more information about usage.

#![feature(conservative_impl_trait)]
#![feature(generator_trait)]
#![feature(try_trait)]
#![feature(use_extern_macros)]
#![feature(never_type)]

extern crate futures_async_macro; // the compiler lies that this has no effect
extern crate futures_await_macro;
extern crate futures;

pub use futures::*;

pub mod prelude {
    pub use {Future, Stream, Sink, Poll, Async, AsyncSink, StartSend};
    pub use {IntoFuture};
    pub use futures_async_macro::{async, async_stream, async_block, async_stream_block};
    pub use futures_await_macro::{await, yeld};
}

/// A hidden module that's the "runtime support" for the async/await syntax.
///
/// The `async` attribute and the `await` macro both assume that they can find
/// this module and use its contents. All of their dependencies are defined or
/// reexported here in one way shape or form.
///
/// This module has absolutely not stability at all. All contents may change at
/// any time without notice. Do not use this module in your code if you wish
/// your code to be stable.
#[doc(hidden)]
pub mod __rt {
    pub use std::boxed::Box;
    pub use std::option::Option::{Some, None};
    pub use std::result::Result::{Ok, Err};
    pub use std::ops::Generator;
    pub use std::marker::PhantomData;

    use futures::Poll;
    use futures::{Future, Async, Stream};
    use std::ops::GeneratorState;
    use std::ops::Try;

    /// Random hack for this causing problems in the compiler's typechecking
    /// pass. Ideally this trait and impl would not be needed.
    ///
    /// ```ignore
    /// -> impl Future<Item = <T as Try>::Ok,
    ///                Error = <T as Try>::Error>
    /// ```
    pub trait MyFuture<T: Try>: Future<Item=T::Ok, Error=T::Error> + 'static {}

    pub trait MyStream<T: Try>: Stream<Item=T::Ok, Error=T::Error> + 'static {}

    impl<F, T> MyFuture<T> for F
        where F: Future<Item = T::Ok, Error = T::Error> + ?Sized + 'static,
              T: Try,
    {}

    impl<F, T> MyStream<T> for F
        where F: Stream<Item = T::Ok, Error = T::Error> + ?Sized + 'static,
              T: Try,
    {}

    // Helper trait to use projections to get access to the `Try` trait's
    // projections without using the `try_trait` feature in consumer crates,
    // just our own crate.
    pub trait MyTry {
        type MyOk;
        type MyError;
    }
    impl<T: Try> MyTry for T {
        type MyOk = T::Ok;
        type MyError = T::Error;
    }

    /// Small shim to translate from a generator to a future or stream.
    ///
    /// This is the translation layer from the generator/coroutine protocol to
    /// the futures protocol.
    struct GenFuture<T, U>(T, PhantomData<U>);

    pub fn gen<T>(t: T) -> impl Future<Item = <T::Return as Try>::Ok,
                                       Error = <T::Return as Try>::Error>
        where T: Generator<Yield = Async<!>>,
              T::Return: Try,
    {
        GenFuture(t, PhantomData::<()>)
    }

    pub fn gen_stream<T, U>(t: T) -> impl Stream<Item = U,
                                       Error = <T::Return as Try>::Error>
        where T: Generator<Yield = Async<U>>,
              T::Return: Try<Ok = ()>,
    {
        GenFuture(t, PhantomData)
    }

    impl<T, U> Future for GenFuture<T, U>
        where T: Generator<Yield = Async<!>>,
              T::Return: Try,
    {
        type Item = <T::Return as Try>::Ok;
        type Error = <T::Return as Try>::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            match self.0.resume() {
                GeneratorState::Yielded(Async::NotReady) => Ok(Async::NotReady),
                GeneratorState::Complete(e) => e.into_result().map(Async::Ready),
            }
        }
    }

    impl<T, U> Stream for GenFuture<T, U>
        where T: Generator<Yield = Async<U>>,
              T::Return: Try<Ok = ()>,
    {
        type Item = U;
        type Error = <T::Return as Try>::Error;

        fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
            match self.0.resume() {
                GeneratorState::Yielded(Async::NotReady) => Ok(Async::NotReady),
                GeneratorState::Yielded(Async::Ready(e)) => Ok(Async::Ready(Some(e))),
                GeneratorState::Complete(e) => e.into_result().map(|()| Async::Ready(None)),
            }
        }
    }
                    
}
