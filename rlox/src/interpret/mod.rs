mod env;
mod error;
mod globals;
mod interpreter;
pub use env::Environment;
pub use error::{RuntimeError, RuntimeErrorKind};
pub use globals::Globals;
pub use interpreter::TreeWalkInterpreter;

pub type Result<T> = std::result::Result<T, RuntimeError>;
