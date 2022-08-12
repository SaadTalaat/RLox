use crate::lex::Token;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum RuntimeCtrl<'a> {
    Error {
        token: &'a Token<'a>,
        line_str: &'a str,
        message: String,
    },
    BreakEmitted,
    ContinueEmitted,
}

impl<'a> RuntimeCtrl<'a> {
    pub fn new(token: &'a Token, message: String) -> Self {
        let line_str = token.line_str;
        Self::Error {
            token,
            line_str,
            message,
        }
    }
}

impl<'a> Error for RuntimeCtrl<'a> {}

impl<'a> Display for RuntimeCtrl<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RuntimeCtrl::Error {
                token,
                line_str,
                message,
            } => {
                write!(f, "Runtime error:\n")?;
                let line_prefix = format!("line({}): ", token.line_index + 1);
                write!(f, "{}{}\n", line_prefix, line_str)?;
                // Then, write a cursor pointing to the offending literal
                let error_msg = {
                    let indent_size = token.offset + line_prefix.len();
                    let indentation: Vec<u8> = (0..indent_size).map(|_| b' ').collect();
                    let indentation = std::str::from_utf8(&indentation).unwrap();
                    format!("{}^ '{}': {}", indentation, token.lexeme(), message)
                };
                write!(f, "{}", error_msg)
            }
            err => panic!("Illegal error propagation: {}", err),
        }
    }
}
