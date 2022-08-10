use super::error::LexicalError;
use std::result;
pub type Result<'a, T> = result::Result<T, LexicalError<'a>>;
