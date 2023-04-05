mod ast;
mod error;
mod rdparser;
pub use ast::{Expr, Operator, Stmt};
pub use rdparser::RDParser;

type Result<T> = std::result::Result<T, error::ParseError>;
