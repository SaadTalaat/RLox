use super::error::RuntimeError;

pub type Result<'a, T> = std::result::Result<T, RuntimeError<'a>>;
