use std::ops::{Generator, GeneratorState};

use futures::task;
use futures::prelude::{Poll, Async, Stream};

use super::{CTX, Reset, IsResult, GenAsyncMove};

pub trait MyStream<T, U: IsResult<Ok=()>>: Stream<Item=T, Error=U::Err> {}

impl<F, T, U> MyStream<T, U> for F
    where F: Stream<Item = T, Error = U::Err> + ?Sized,
          U: IsResult<Ok=()>
{}

impl<Gen, Yield> Stream for GenAsyncMove<Gen, Yield>
    where Gen: Generator<Yield = Async<Yield>>,
          Gen::Return: IsResult<Ok = ()>,
{
    type Item = Yield;
    type Error = <Gen::Return as IsResult>::Err;

    fn poll_next(&mut self, ctx: &mut task::Context) -> Poll<Option<Self::Item>, Self::Error> {
        CTX.with(|cell| {
            let _r = Reset::new(ctx, cell);
            if self.done { return Ok(Async::Ready(None)) }
            // Because we are controlling the creation of our underlying
            // generator, we know that this is definitely a movable generator
            // so calling resume is always safe.
            match unsafe { self.gen.resume() } {
                GeneratorState::Yielded(Async::Ready(e)) => {
                    Ok(Async::Ready(Some(e)))
                }
                GeneratorState::Yielded(Async::Pending) => {
                    Ok(Async::Pending)
                }
                GeneratorState::Complete(e) => {
                    self.done = true;
                    e.into_result().map(|()| Async::Ready(None))
                }
            }
        })
    }
}
