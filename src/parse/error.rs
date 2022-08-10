use crate::lex::Token;
use std::error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct ParseError<'a> {
    token: &'a Token<'a>,
    line_str: &'a str,
    message: String,
}

impl<'a> ParseError<'a> {
    pub fn new(token: &'a Token, message: String) -> Self {
        let line_str = token.line_str;
        Self {
            token,
            line_str,
            message,
        }
    }
}

impl<'a> error::Error for ParseError<'a> {}
impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Parse error:\n")?;
        let line_prefix = format!("line({}): ", self.token.line_index + 1);
        write!(f, "{}{}\n", line_prefix, self.line_str)?;
        // Then, write a cursor pointing to the offending literal
        let error_msg = {
            let indent_size = self.token.offset + line_prefix.len();
            let indentation: Vec<u8> = (0..indent_size).map(|_| b' ').collect();
            let indentation = std::str::from_utf8(&indentation).unwrap();
            format!("{}^ {}: {}", indentation, self.token.lexeme(), self.message)
        };
        write!(f, "{}", error_msg)
    }
}
