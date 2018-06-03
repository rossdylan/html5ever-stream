use hyper;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Hyper(#[cause] hyper::Error),
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Hyper(e)
    }
}
