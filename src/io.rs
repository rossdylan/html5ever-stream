use std::io;
use html5ever::{
    parse_document,
    Parser,
    tree_builder::TreeSink,
    tendril::TendrilSink,
    tendril::stream::Utf8LossyDecoder,
};


/// ParserSink is a simple wrapper around a html5ever parser. It implements
/// `std::io::Write` and allows you to stream data into it via the `std::io` primitives
pub struct ParserSink<D: TreeSink> {
    inner: Utf8LossyDecoder<Parser<D>>,
}

impl<D> ParserSink<D> where D: TreeSink {
    /// new creates a new html5ever parser and wraps it in a structure that implements
    /// `std::io::Write`
    pub fn new(dom: D) -> Self {
        let parser = parse_document(dom, Default::default()).from_utf8();
        return ParserSink{
            inner: parser,
        }
    }

    /// finish comsumes the ParserSink and returns the document structure completed by
    /// the inner parser.
    pub fn finish(self) -> D::Output {
        self.inner.finish()
    }
}

impl<D> io::Write for ParserSink<D> where D: TreeSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.process(buf.into());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate reqwest;
    use html5ever::rcdom::RcDom;
    use std::io::Write;

    use super::*;
    const TEST_HTML: &'static str = "<html> <head> <title> test </title> </head> </html>";
    #[test]
    fn test_write() {
        let mut ps = ParserSink::new(RcDom::default());
        assert_eq!(ps.write(TEST_HTML.as_bytes()).unwrap(), TEST_HTML.len());
        ps.finish();
    }
}
