pub mod error;
pub mod result;
pub mod scanner;
mod token;
pub use result::Result;
pub use scanner::Scanner;
pub use token::{LiteralValue, Token, TokenType};

#[cfg(test)]
mod tests;
