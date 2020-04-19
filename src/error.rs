use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;
use std::result;

#[derive(Debug)]
pub struct KvsError {
    inner: Context<KvsErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum KvsErrorKind {
    #[fail(display = "ConnectionError")]
    ConnectionError,

    #[fail(display = "Data too short")]
    DataTooShort(usize),

    #[fail(display = "Invalid Command.")]
    InvalidCommand,

    #[fail(display = "Invalid Data.")]
    InvalidData,

    #[fail(display = "Invalid Engine.")]
    InvalidEngine,

    #[fail(display = "Invalid prefix")]
    InvalidPrefix(u8),

    #[fail(display = "Key not found")]
    KeyNotFound,

    #[fail(display = "Error with log file")]
    FileError,

    #[fail(display = "Error parsing")]
    ParsingError,

    #[fail(display = "Error from sled crate")]
    SledError,

    #[fail(display = "Uncompatible Engine")]
    UncompatibleEngine,

    #[fail(display = "An unknown error has occurred.")]
    UnknownError,
}

impl Fail for KvsError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl KvsError {
    pub fn kind(&self) -> KvsErrorKind {
        *self.inner.get_context()
    }
}

impl From<KvsErrorKind> for KvsError {
    fn from(kind: KvsErrorKind) -> KvsError {
        KvsError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<KvsErrorKind>> for KvsError {
    fn from(inner: Context<KvsErrorKind>) -> KvsError {
        KvsError { inner }
    }
}

pub type Result<T> = result::Result<T, KvsError>;
pub use KvsError as Error;
pub use KvsErrorKind as ErrorKind;
