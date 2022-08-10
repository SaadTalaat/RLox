mod error;
mod result;
mod scanner;
mod token;

pub use error::LexicalError;
pub use result::Result;
pub use scanner::Scanner;
pub use token::{Token, TokenType};

#[cfg(test)]
mod tests;
