mod ast;
mod error;
mod rdparser;
mod resolver;
pub use ast::{Expr, Operator, Stmt};
pub use rdparser::RDParser;
pub use resolver::Resolver;

type Result<T> = std::result::Result<T, error::ParseError>;
