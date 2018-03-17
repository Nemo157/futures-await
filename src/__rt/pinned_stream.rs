use std::ops::{Generator, GeneratorState};

use __rt::pin_api::mem::Pin;

use futures::task;
use futures::prelude::{Poll, Async};
use futures::stable::StableStream;

use super::{CTX, Reset, IsResult, GenAsync};

pub trait MyStableStream<T, U: IsResult<Ok=()>>: StableStream<Item=T, Error=U::Err> {}

impl<F, T, U> MyStableStream<T, U> for F
    where F: StableStream<Item = T, Error = U::Err> + ?Sized,
          U: IsResult<Ok=()>
{}

impl<T, U> StableStream for GenAsync<T, U>
    where T: Generator<Yield = Async<U>>,
          T::Return: IsResult<Ok = ()>,
{
    type Item = U;
    type Error = <T::Return as IsResult>::Err;

    fn poll_next(mut self: Pin<Self>, ctx: &mut task::Context) -> Poll<Option<Self::Item>, Self::Error> {
        CTX.with(|cell| {
            let _r = Reset::new(ctx, cell);
            let this: &mut Self = unsafe { Pin::get_mut(&mut self) };
            if this.done { return Ok(Async::Ready(None)) }
            match this.gen.resume() {
                GeneratorState::Yielded(Async::Ready(e)) => {
                    Ok(Async::Ready(Some(e)))
                }
                GeneratorState::Yielded(Async::Pending) => {
                    Ok(Async::Pending)
                }
                GeneratorState::Complete(e) => {
                    this.done = true;
                    e.into_result().map(|()| Async::Ready(None))
                }
            }
        })
    }
}
