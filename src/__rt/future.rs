use std::ops::{Generator, GeneratorState};

use super::{IsResult, Reset, CTX, GenAsyncMove};

use futures::Never;
use futures::task;
use futures::prelude::{Poll, Async, Future};

pub trait MyFuture<T: IsResult>: Future<Item=T::Ok, Error = T::Err> {}

impl<F, T> MyFuture<T> for F
    where F: Future<Item = T::Ok, Error = T::Err > + ?Sized,
          T: IsResult
{}

impl<Gen> Future for GenAsyncMove<Gen, Never>
    where Gen: Generator<Yield = Async<Never>>,
          Gen::Return: IsResult,
{
    type Item = <Gen::Return as IsResult>::Ok;
    type Error = <Gen::Return as IsResult>::Err;

    fn poll(&mut self, ctx: &mut task::Context) -> Poll<Self::Item, Self::Error> {
        CTX.with(|cell| {
            let _r = Reset::new(ctx, cell);
            // Because we are controlling the creation of our underlying
            // generator, we know that this is definitely a movable generator
            // so calling resume is always safe.
            match unsafe { self.gen.resume() } {
                GeneratorState::Yielded(Async::Pending)
                    => Ok(Async::Pending),
                GeneratorState::Yielded(Async::Ready(mu))
                    => match mu {},
                GeneratorState::Complete(e)
                    => e.into_result().map(Async::Ready),
            }
        })
    }
}

