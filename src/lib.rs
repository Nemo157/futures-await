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
#![feature(use_extern_macros)]
#![feature(never_type)]
#![feature(on_unimplemented)]

extern crate futures_async_macro; // the compiler lies that this has no effect
extern crate futures_await_macro;
extern crate futures;

pub use futures::*;

pub mod prelude {
    pub use {Future, Stream, Sink, Poll, Async, AsyncSink, StartSend};
    pub use {IntoFuture};
    pub use futures_async_macro::{async, async_block};
    pub use futures_await_macro::{await, stream_yield};
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
    pub use std::result::Result::{Ok, Err, self};
    pub use std::ops::Generator;
    pub use std::marker::PhantomData;

    use futures::Poll;
    use futures::{Future, Async, Stream};
    use std::ops::GeneratorState;

    pub trait MyFuture<T: IsResult>: Future<Item=T::Ok, Error = T::Err> {}

    pub trait MyStream<T, U: IsResult<Ok=()>>: Stream<Item=T, Error=U::Err> {}

    impl<F, T> MyFuture<T> for F
        where F: Future<Item = T::Ok, Error = T::Err > + ?Sized,
              T: IsResult
    {}

    impl<F, T, U> MyStream<T, U> for F
        where F: Stream<Item = T, Error = U::Err> + ?Sized,
              U: IsResult<Ok=()>
    {}

    #[rustc_on_unimplemented = "async functions must return a `Result` or \
                                a typedef of `Result`"]
    pub trait IsResult {
        type Ok;
        type Err;

        fn into_result(self) -> Result<Self::Ok, Self::Err>;
    }
    impl<T, E> IsResult for Result<T, E> {
        type Ok = T;
        type Err = E;

        fn into_result(self) -> Result<Self::Ok, Self::Err> { self }
    }

    pub trait IsAsync {
        type Item;

        fn into_async(self) -> Async<Self::Item>;
    }
    impl<T> IsAsync for Async<T> {
        type Item = T;

        fn into_async(self) -> Async<Self::Item> { self }
    }

    pub fn diverge<T>() -> T { loop {} }

    /// Small shim to translate from a generator to a future.
    ///
    /// This is the translation layer from the generator/coroutine protocol to
    /// the futures protocol.
    pub struct GenFuture<T>(pub T);

    impl<T> Future for GenFuture<T>
        where T: Generator,
              T::Yield: IsAsync<Item=!>,
              T::Return: IsResult,
    {
        type Item = <T::Return as IsResult>::Ok;
        type Error = <T::Return as IsResult>::Err;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            match self.0.resume() {
                GeneratorState::Yielded(a) => match a.into_async() {
                    Async::NotReady => Ok(Async::NotReady),
                },
                GeneratorState::Complete(e) => e.into_result().map(Async::Ready),
            }
        }
    }

    impl<T> Stream for GenFuture<T>
        where T: Generator,
              T::Yield: IsAsync,
              T::Return: IsResult<Ok = ()>,
    {
        type Item = <T::Yield as IsAsync>::Item;
        type Error = <T::Return as IsResult>::Err;

        fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
            match self.0.resume() {
                GeneratorState::Yielded(a) => Ok(a.into_async().map(Some)),
                GeneratorState::Complete(e) => e.into_result().map(|()| Async::Ready(None)),
            }
        }
    }
}
