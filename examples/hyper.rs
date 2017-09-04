#![feature(proc_macro, conservative_impl_trait, generators)]

#![allow(unused_doc_comment)]

extern crate futures_await as futures;
extern crate tokio_core;
extern crate hyper;
#[macro_use]
extern crate error_chain;

use std::io;

use futures::{Future, Stream};
use futures::prelude::{async, await};
use tokio_core::reactor::Core;
use hyper::Client;
use hyper::client::HttpConnector;

error_chain! {
    foreign_links {
        Io(io::Error);
        Hyper(hyper::Error);
        HyperUriError(hyper::error::UriError);
        FromUtf8Error(std::string::FromUtf8Error);
    }
}

fn main() {
    // Create the event loop that will drive this server
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    let result = fetch(client);
    println!("{}", core.run(result).unwrap());
}

#[async]
fn fetch(client: Client<HttpConnector>) -> impl Future<Item=String, Error=Error> {
    let response = await!(client.get("http://httpbin.org/headers".parse()?))?;
    if !response.status().is_success() {
        bail!(io::Error::new(io::ErrorKind::Other, "request failed"));
    }
    let body = await!(response.body().concat2())?;
    let string = String::from_utf8(body.as_ref().to_owned())?;
    Ok(string)
}
