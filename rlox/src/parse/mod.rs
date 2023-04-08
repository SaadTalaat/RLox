mod ast;
mod error;
mod rdparser;
mod resolver;
pub use ast::{Expr, ExprKind, Operator, Stmt, StmtKind};
pub use rdparser::RDParser;
pub use resolver::Resolver;

type Result<T> = std::result::Result<T, error::ParseError>;
