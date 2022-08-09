pub mod error;
pub mod result;
pub mod scanner;
mod token;
pub use scanner::Scanner;
pub use token::{LiteralValue, Token, TokenType};
pub use result::Result;

#[cfg(test)]
mod tests;
