use super::error::ParseError;

pub type Result<'a, T> = std::result::Result<T, ParseError<'a>>;
