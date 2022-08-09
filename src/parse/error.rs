use crate::lex::Token;
use std::error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct ParseError {
    line: usize,
    message: String,
}

impl ParseError {
    pub fn new(token: &Token, message: String) -> Self {
        Self {
            line: token.line,
            message,
        }
    }
}

impl error::Error for ParseError {}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}\nat line {}", self.message, self.line)
    }
}
