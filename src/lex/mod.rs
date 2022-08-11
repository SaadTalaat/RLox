mod error;
mod lexer;
mod result;
mod token;

pub use error::LexicalError;
pub use lexer::Lexer;
pub use result::Result;
pub use token::{Token, TokenType};

#[cfg(test)]
mod tests;
