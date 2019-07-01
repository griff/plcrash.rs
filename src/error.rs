use std::fmt::{self, Display};
use std::path::PathBuf;

use failure::{Backtrace, Context, Fail};

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    // A plain enum with no data in any of its variants
    //
    // For example:
    #[fail(display = "A contextual error message.")]
    OneVariant,
    // ...
    #[fail(display = "error parsing dSYM {:?} in zip {:?}", _1, _0)]
    DSYM(PathBuf, PathBuf),
    #[fail(display = "error loading zip {:?}", _0)]
    Zip(PathBuf),
    #[fail(display = "error looking up {:x} in {:?}", _1, _0)]
    Probe(PathBuf, u64),
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Context::new(kind) }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}
