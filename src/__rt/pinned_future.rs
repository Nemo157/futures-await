use std::mem::Pin;
use std::ops::{Generator, GeneratorState};

use super::{IsResult, Reset, CTX, GenAsync};

use futures::Never;
use futures::stable::StableFuture;
use task;
use prelude::{Poll, Async};

pub trait MyStableFuture<T: IsResult>: StableFuture<Item=T::Ok, Error = T::Err> {}

impl<F, T> MyStableFuture<T> for F
    where F: StableFuture<Item = T::Ok, Error = T::Err> + ?Sized,
          T: IsResult,
{}

impl<Gen> StableFuture for GenAsync<Gen, Never>
    where Gen: Generator<Yield = Async<Never>>,
          Gen::Return: IsResult,
{
    type Item = <Gen::Return as IsResult>::Ok;
    type Error = <Gen::Return as IsResult>::Err;

    fn poll(mut self: Pin<Self>, ctx: &mut task::Context) -> Poll<Self::Item, Self::Error> {
        CTX.with(|cell| {
            let _r = Reset::new(ctx, cell);
            let this: &mut Self = unsafe { Pin::get_mut(&mut self) };
            // This is an immovable generator, but since we're only accessing
            // it via a Pin this is safe.
            match unsafe { this.gen.resume() } {
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
