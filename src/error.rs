use super::lex;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub struct Error(String);

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Error {
        Error(format!("{}", other))
    }
}

impl From<fmt::Error> for Error {
    fn from(other: fmt::Error) -> Error {
        Error(format!("{}", other))
    }
}

impl From<lex::error::Error<'_>> for Error {
    fn from(other: lex::error::Error) -> Error {
        Error(format!("{}", other))
    }
}
pub type Result<T> = std::result::Result<T, Error>;
