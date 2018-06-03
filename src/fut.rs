use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;

use futures::{Future, Stream, Poll, Async};
use html5ever::{
    parse_document,
    Parser,
    rcdom,
    rcdom::RcDom,
    tree_builder::TreeSink,
    tendril::TendrilSink,
    tendril::stream::Utf8LossyDecoder,
};

use errors;

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
/// use html5ever_stream::fut::ParserFuture;
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
          E: Into<errors::Error>,
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
          E: Into<errors::Error>,
          D: TreeSink,
{
    type Item = D::Output;
    type Error = errors::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match self.state {
                ParserState::Parsing(ref mut parser) => match self.stream.poll().map_err(|e| e.into())? {
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


/// NodeStream uses a VecDeque to fully traverse the given RcDom and emit reference
/// counted handles to each node as a `futures::Stream`. Pretty sure this won't leak
/// memory since everything is either owned by a NodeStream struct or Rc'd.
/// TODO(rossdylan) Actually verify that this doesn't leak
/// # Examples
/// ```rust
/// extern crate html5ever;
/// extern crate hyper;
/// extern crate html5ever_stream;
/// extern crate futures;
///
/// use futures::{Future, Stream};
/// use html5ever_stream::fut::{ParserFuture, NodeStream};
/// use html5ever::rcdom::{RcDom, NodeData};
/// use hyper::Body;
///
/// const TEST_HTML: &'static str = "<html> <head> <title> test </title> </head> </html>";
/// let body: Body = TEST_HTML.into();
/// let dom = ParserFuture::new(body, RcDom::default()).wait().unwrap();
/// NodeStream::new(dom).for_each(|n| {
///     match &n.data {
///         NodeData::Element { ref name, .. } => {
///             println!("elemn: {}", name.local);
///         },
///         _ => {},
///     };
///     Ok(())
/// }).wait();
/// ```
pub struct NodeStream {
    _dom: RcDom,
    queue: VecDeque<rcdom::Handle>,
}

impl NodeStream {
    /// new will create a new NodeStream from the provided RcDom struct. We take
    /// ownership of the RcDom and store it internally. We then seed our iteration
    /// by adding a Rc reference to the first node to our queue.
    pub fn new(dom: RcDom) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(Rc::clone(&dom.document));
        NodeStream{
            _dom: dom,
            queue: queue,
        }
    }
}

impl Stream for NodeStream {
    type Item = rcdom::Handle;
    type Error = errors::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.queue.pop_front() {
            Some(ref handle) => {
                for child in handle.children.borrow().iter() {
                    self.queue.push_back(Rc::clone(child));
                }
                Ok(Async::Ready(Some(Rc::clone(handle))))
            },
            None => {
                Ok(Async::Ready(None))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use hyper::Body;
    use html5ever::rcdom::RcDom;

    use super::*;
    const TEST_HTML: &'static str = "<html> <head> <title> test </title> </head> </html>";
    #[test]
    fn test_hyper_body_stream() {
        let body: Body = TEST_HTML.into();
        let pf = ParserFuture::new(body, RcDom::default());
        let res = pf.wait();
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn test_basic_node_stream() {
        let body: Body = TEST_HTML.into();
        let pf = ParserFuture::new(body, RcDom::default());
        let res = pf.wait();
        assert_eq!(res.is_ok(), true);
        let dom = res.unwrap();

        let stream = NodeStream::new(dom);
        let res = stream.collect().wait();
        assert_eq!(res.is_ok(), true);
        assert_eq!(res.unwrap().len(), 9);
    }
}
