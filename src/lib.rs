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
#![feature(on_unimplemented)]

extern crate futures_await_async_macro as async_macro;
extern crate futures_await_await_macro as await_macro;
extern crate futures;

pub use futures::*;

pub mod prelude {
    pub use futures::prelude::*;
    pub use async_macro::{async, async_block};
    pub use await_macro::{await, stream_yield};
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
    pub extern crate std;
    pub use std::ops::Generator;

    use futures::Poll;
    use futures::{Future, Async, Stream};
    use std::ops::GeneratorState;
    use std::marker::PhantomData;

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

    pub fn diverge<T>() -> T { loop {} }

    /// Small shim to translate from a generator to a future or stream.
    ///
    /// This is the translation layer from the generator/coroutine protocol to
    /// the futures protocol.
    pub struct GenFuture<U, T> {
        gen: T,
        done: bool,
        phantom: PhantomData<U>,
    }

    /// Uninhabited type to allow `await!` to work across both `async` and
    /// `async_stream`.
    pub enum Mu {}

    pub fn gen<T, U>(gen: T) -> GenFuture<U, T>
        where T: Generator<Yield = Async<U>>,
              T::Return: IsResult,
    {
        GenFuture { gen, done: false, phantom: PhantomData }
    }

    impl<T> Future for GenFuture<Mu, T>
        where T: Generator<Yield = Async<Mu>>,
              T::Return: IsResult,
    {
        type Item = <T::Return as IsResult>::Ok;
        type Error = <T::Return as IsResult>::Err;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            match self.gen.resume() {
                GeneratorState::Yielded(Async::NotReady)
                    => Ok(Async::NotReady),
                GeneratorState::Yielded(Async::Ready(mu))
                    => match mu {},
                GeneratorState::Complete(e)
                    => e.into_result().map(Async::Ready),
            }
        }
    }

    impl<U, T> Stream for GenFuture<U, T>
        where T: Generator<Yield = Async<U>>,
              T::Return: IsResult<Ok = ()>,
    {
        type Item = U;
        type Error = <T::Return as IsResult>::Err;

        fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
            if self.done { return Ok(Async::Ready(None)) }
            match self.gen.resume() {
                GeneratorState::Yielded(Async::Ready(e)) => {
                    Ok(Async::Ready(Some(e)))
                }
                GeneratorState::Yielded(Async::NotReady) => {
                    Ok(Async::NotReady)
                }
                GeneratorState::Complete(e) => {
                    self.done = true;
                    e.into_result().map(|()| Async::Ready(None))
                }
            }
        }
    }
}
