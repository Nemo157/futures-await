mod future;
mod stream;
mod pinned_future; mod pinned_stream;

use std::cell::Cell;
use std::mem;
use std::ptr;
use futures::task;
use std::marker::{PhantomData, Unpin};

pub use self::future::*;
pub use self::stream::*;
pub use self::pinned_future::*;
pub use self::pinned_stream::*;

pub use futures::prelude::{Async, Future, Stream};
pub use futures::stable::{StableFuture, StableStream};

pub extern crate std;

pub use std::ops::Generator;

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

type StaticContext = *mut task::Context<'static>;

thread_local!(static CTX: Cell<StaticContext> = Cell::new(ptr::null_mut()));

struct Reset<'a>(StaticContext, &'a Cell<StaticContext>);

impl<'a> Reset<'a> {
    fn new(ctx: &mut task::Context, cell: &'a Cell<StaticContext>) -> Reset<'a> {
        let stored_ctx = unsafe { mem::transmute::<&mut task::Context, StaticContext>(ctx) };
        let ctx = cell.replace(stored_ctx);
        Reset(ctx, cell)
    }

    fn new_null(cell: &'a Cell<StaticContext>) -> Reset<'a> {
        let ctx = cell.replace(ptr::null_mut());
        Reset(ctx, cell)
    }
}

impl<'a> Drop for Reset<'a> {
    fn drop(&mut self) {
        self.1.set(self.0);
    }
}

pub fn in_ctx<F: FnOnce(&mut task::Context) -> T, T>(f: F) -> T {
    CTX.with(|cell| {
        let r = Reset::new_null(cell);
        if r.0 == ptr::null_mut() {
            panic!("Cannot use `await!`, `await_item!`, or `#[async] for` outside of an `async` function.")
        }
        f(unsafe { &mut *r.0 })
    })
}

/// Small shim to translate from a generator to a future or stream.
///
/// This is the translation layer from the generator/coroutine protocol to
/// the futures protocol.
pub struct GenAsync<Gen, Yield> {
    gen: Gen,
    done: bool,
    phantom: PhantomData<Yield>,
}

/// Small shim to translate from a generator to a future or stream.
///
/// This is the translation layer from the generator/coroutine protocol to
/// the futures protocol.
pub struct GenAsyncMove<Gen, Yield> {
    gen: Gen,
    done: bool,
    phantom: PhantomData<Yield>,
}

impl<Gen, Yield> !Unpin for GenAsync<Gen, Yield> {
}

pub fn gen_async<Gen, Yield>(gen: Gen) -> GenAsync<Gen, Yield>
    where Gen: Generator<Yield = Async<Yield>>, Gen::Return: IsResult,
{
    GenAsync { gen, done: false, phantom: PhantomData }
}

pub fn gen_async_move<Gen, Yield>(gen: Gen) -> GenAsyncMove<Gen, Yield>
    where Gen: Generator<Yield = Async<Yield>>, Gen::Return: IsResult,
{
    GenAsyncMove { gen, done: false, phantom: PhantomData }
}
