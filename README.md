# html5ever-stream
[![Travis CI Status](https://travis-ci.org/rossdylan/html5ever-stream.svg?branch=master)](https://travis-ci.org/rossdylan/html5ever-stream)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/crates/v/html5ever-stream.svg)](https://crates.io/crates/html5ever-stream)
[![Released API docs](https://docs.rs/html5ever-stream/badge.svg)](https://docs.rs/html5ever-stream)

Adapters to easily stream data into an [html5ever](https://crates.io/crates/html5ever) parser.

## Overview
This crate aims to provide shims to make it relatively painless to parse html from some stream of data.
This stream could be consumed by the standard IO Reader/Writer traits, or via a [Stream](https://docs.rs/futures/0.1.2/futures/stream/trait.Stream.html) from the [futures](https://docs.rs/futures/0.1.21/futures/) crate

* Support for any Stream that emits an item implementing AsRef<[u8]>
    * Supports hyper and unstable reqwest types automatically
* Support for [reqwest's copy_to](https://docs.rs/reqwest/0.8.6/reqwest/struct.Response.html#method.copy_to) method
* Helper wrappers for RcDom to make it easier to work with.

## Examples

### Using Hyper 0.11

```rust
extern crate futures;
extern crate html5ever;
extern crate html5ever_stream;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate num_cpus;

use html5ever::rcdom;
use futures::{Future, Stream};
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use html5ever_stream::{ParserFuture, NodeStream};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(num_cpus::get(), &handle).unwrap())
        .build(&handle);


    // NOTE: We throw away errors here in two places, you are better off casting them into your
    // own custom error type in order to propagate them. I believe async/await will also help here.
    let req_fut = client.get("https://github.com".parse().unwrap()).map_err(|_| ());
    let parser_fut = req_fut.and_then(|res| ParserFuture::new(res.body().map_err(|_| ()), rcdom::RcDom::default()));
    let nodes = parser_fut.and_then(|dom| {
        NodeStream::new(&dom).collect()
    });
    let print_fut = nodes.and_then(|vn| {
        println!("found {} elements", vn.len());
        Ok(())
    });
    core.run(print_fut).unwrap();
}
```

### Using Unstable Async Reqwest 0.8.6

```rust
extern crate futures;
extern crate html5ever;
extern crate html5ever_stream;
extern crate reqwest;
extern crate tokio_core;

use html5ever::rcdom;
use futures::{Future, Stream};
use reqwest::unstable::async as async_reqwest;
use tokio_core::reactor::Core;
use html5ever_stream::{ParserFuture, NodeStream};

fn main() {
    let mut core = Core::new().unwrap();
    let client = async_reqwest::Client::new(&core.handle());

    // NOTE: We throw away errors here in two places, you are better off casting them into your
    // own custom error type in order to propagate them. I believe async/await will also help here.
    let req_fut = client.get("https://github.com").send().map_err(|_| ());
    let parser_fut = req_fut.and_then(|res| ParserFuture::new(res.into_body().map_err(|_| ()), rcdom::RcDom::default()));
    let nodes = parser_fut.and_then(|dom| {
        NodeStream::new(&dom).collect()
    });
    let print_fut = nodes.and_then(|vn| {
        println!("found {} elements", vn.len());
        Ok(())
    });
    core.run(print_fut).unwrap();
}
```

### Using Stable Reqwest 0.8.6

```rust
extern crate html5ever;
extern crate html5ever_stream;
extern crate reqwest;

use html5ever::rcdom;
use html5ever_stream::{ParserSink, NodeIter};

fn main() {
    let mut resp = reqwest::get("https://github.com").unwrap();
    let mut parser = ParserSink::new(rcdom::RcDom::default());
    resp.copy_to(&mut parser).unwrap();
    let document = parser.finish();
    let nodes: Vec<rcdom::Handle> = NodeIter::new(&document).collect();
    println!("found {} elements", nodes.len());
}
```


## License
Licensed under the [MIT License](http://opensource.org/licenses/MIT)

