mod error;
mod lexer;
mod token;

// exported types
pub use lexer::Lexer;
pub use token::{Token, TokenType};

pub type Result<T> = std::result::Result<T, error::LexicalError>;
