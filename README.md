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
use html5ever_stream::fut::{ParserFuture, NodeStream};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(num_cpus::get(), &handle).unwrap())
        .build(&handle);


    let work = client.get("https://github.com".parse().unwrap()).and_then(|res| {
        let parser = ParserFuture::new(res.body(), rcdom::RcDom::default());
        parser.and_then(|dom| {
            NodeStream::new(dom).for_each(|n| {
                match &n.data {
                    rcdom::NodeData::Element { ref name, .. } => {
                        println!("elem: {}", name.local);
                    },
                    _ => {},
                }
                Ok(())
            // TODO(rossdylan) This is a kludge fix it
            }).map_err(|_| hyper::Error::Closed)
        })
    });
    core.run(work).unwrap();
}
```


## License
Licensed under the [MIT License](http://opensource.org/licenses/MIT)

