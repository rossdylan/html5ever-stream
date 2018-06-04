use std::marker::PhantomData;
use std::mem;

use futures::{Future, Stream, Poll, Async};
use html5ever::{
    parse_document,
    Parser,
    tree_builder::TreeSink,
    tendril::TendrilSink,
    tendril::stream::Utf8LossyDecoder,
};

enum ParserState<D: TreeSink> {
    Parsing(Utf8LossyDecoder<Parser<D>>),
    Finished
}

/// ParserFuture takes in any stream that emits an item that can be referenced as a `[u8]`
/// It will collect the data from that stream into a html5ever parser. Currently you can't
/// control the parser, but eventually you will. The future resolves to a RcDom structure.
/// # Examples
/// ```rust
/// extern crate html5ever;
/// extern crate hyper;
/// extern crate html5ever_stream;
/// extern crate futures;
///
/// use futures::Future;
/// use html5ever_stream::ParserFuture;
/// use html5ever::rcdom::RcDom;
/// use hyper::Body;
///
/// const TEST_HTML: &'static str = "<html> <head> <title> test </title> </head> </html>";
/// let body: Body = TEST_HTML.into();
/// let dom = ParserFuture::new(body, RcDom::default()).wait().unwrap();
/// ```
#[must_use = "streams do nothing unless polled"]
pub struct ParserFuture<S, C, E, D> 
    where D: TreeSink,
{
    stream: S,
    state: ParserState<D>,
    body_type: PhantomData<C>,
    err_type: PhantomData<E>,
}

impl<S, C, E, D> ParserFuture<S, C, E, D>
    where S: Stream<Item=C, Error=E>,
          C: AsRef<[u8]>,
          D: TreeSink,
{

    pub fn new(s: S, dom: D) -> ParserFuture<S, C, E, D> {
        let parser = parse_document(dom, Default::default()).from_utf8();

        ParserFuture {
            stream: s,
            state: ParserState::Parsing(parser),
            body_type: PhantomData,
            err_type: PhantomData,
        }
    }
}

impl<S, C, E, D> Future for ParserFuture<S, C, E, D>
    where S: Stream<Item=C, Error=E>,
          C: AsRef<[u8]>,
          D: TreeSink,
{
    type Item = D::Output;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match self.state {
                ParserState::Parsing(ref mut parser) => match self.stream.poll()? {
                    Async::Ready(Some(chunk)) => {
                        parser.process(chunk.as_ref().into());
                        continue;
                    },
                    Async::Ready(None) => {},
                    Async::NotReady => return Ok(Async::NotReady),
                },
                ParserState::Finished => panic!("Polled completed Parser"),
            };
            match mem::replace(&mut self.state, ParserState::Finished) {
                ParserState::Parsing(parser) => {
                    return Ok(Async::Ready(parser.finish()))
                },
                ParserState::Finished => panic!(),
            }
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate hyper;
    extern crate reqwest;
    extern crate futures;
    use futures::{Future, Stream};
    use self::reqwest::unstable::async;
    use html5ever::rcdom::RcDom;
    use ::{ParserFuture, NodeStream};

    const TEST_HTML: &'static str = "<html> <head> <title> test </title> </head> </html>";
    #[test]
    fn test_hyper_body_stream() {
        let body: hyper::Body = TEST_HTML.into();
        let pf = ParserFuture::new(body, RcDom::default());
        let res = pf.wait();
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn test_basic_hyper_node_stream() {
        let body: hyper::Body = TEST_HTML.into();
        let pf = ParserFuture::new(body, RcDom::default());
        let res = pf.wait();
        assert_eq!(res.is_ok(), true);
        let dom = res.unwrap();

        let stream = NodeStream::new(&dom);
        let res = stream.collect().wait();
        assert_eq!(res.is_ok(), true);
        assert_eq!(res.unwrap().len(), 9);
    }

    /// This test is basically a noop, but it does check that all the types work out
    /// Eventually when the reqwest async impl becomes stable we should be able to
    /// properly test it.
    #[test]
    fn test_reqwest_body_stream() {
        let pf = ParserFuture::new(async::Decoder::empty(), RcDom::default());
        let res = pf.wait();
        assert_eq!(res.is_ok(), true);
    }
}
