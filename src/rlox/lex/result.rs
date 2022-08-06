use super::error::Error;
use std::result;
pub type Result<'a, T> = result::Result<T, Error<'a>>;
