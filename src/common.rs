use std::collections::VecDeque;
use std::rc::Rc;

use futures::{Stream, Poll, Async};
use html5ever::rcdom;

pub struct NodeTraverser {
    queue: VecDeque<rcdom::Handle>,
}

impl NodeTraverser {
    fn new(dom: &rcdom::RcDom) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(Rc::clone(&dom.document));
        NodeTraverser{
            queue: queue,
        }
    }

    fn next(&mut self) -> Option<rcdom::Handle> {
        match self.queue.pop_front() {
            Some(ref handle) => {
                for child in handle.children.borrow().iter() {
                    self.queue.push_back(Rc::clone(child));
                }
                Some(Rc::clone(handle))
            },
            None => None,
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
/// use html5ever_stream::{ParserFuture, NodeStream};
/// use html5ever::rcdom::{RcDom, NodeData};
/// use hyper::Body;
///
/// const TEST_HTML: &'static str = "<html> <head> <title> test </title> </head> </html>";
/// let body: Body = TEST_HTML.into();
/// let dom = ParserFuture::new(body, RcDom::default()).wait().unwrap();
/// NodeStream::new(&dom).for_each(|n| {
///     match &n.data {
///         NodeData::Element { ref name, .. } => {
///             println!("elemn: {}", name.local);
///         },
///         _ => {},
///     };
///     Ok(())
/// }).wait();
/// ```
pub struct NodeStream(NodeTraverser);

impl NodeStream {
    pub fn new(dom: &rcdom::RcDom) -> Self {
        NodeStream(NodeTraverser::new(dom))
    }
}

impl Stream for NodeStream {
    type Item = rcdom::Handle;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(self.0.next()))
    }
}

pub struct NodeIter(NodeTraverser);

impl NodeIter {
    pub fn new(dom: &rcdom::RcDom) -> Self {
        NodeIter(NodeTraverser::new(dom))
    }
}

impl Iterator for NodeIter {
    type Item = rcdom::Handle;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
