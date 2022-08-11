use std::error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct LexicalError<'a> {
    line_index: usize,
    offset: usize,
    illegal_literal: char,
    line: &'a str,
    message: &'a str,
}

impl<'a> LexicalError<'a> {
    pub fn new(
        line_index: usize,
        offset: usize,
        illegal_literal: char,
        source: &'a [u8],
        message: &'a str,
    ) -> Self {
        let line = std::str::from_utf8(source)
            .unwrap()
            .lines()
            .collect::<Vec<&str>>()[line_index];
        Self {
            line_index,
            offset,
            illegal_literal,
            line: line,
            message,
        }
    }

    pub fn copied(other: &Self) -> Self {
        Self {
            line_index: other.line_index,
            offset: other.offset,
            illegal_literal: other.illegal_literal,
            line: other.line,
            message: other.message,
        }
    }
}

impl<'a> error::Error for LexicalError<'a> {}

impl<'a> Display for LexicalError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // First, write the offending line to the output.
        write!(f, "Lexical error:\n")?;
        let line_prefix = format!("line({}): ", self.line_index + 1);
        write!(f, "{}{}\n", line_prefix, self.line)?;
        // Then, write a cursor pointing to the offending literal
        let error_msg = {
            let indent_size = self.offset + line_prefix.len();
            let indentation: Vec<u8> = (0..indent_size).map(|_| b' ').collect();
            let indentation = std::str::from_utf8(&indentation).unwrap();
            format!(
                "{}^ '{}': {}",
                indentation, self.illegal_literal, self.message
            )
        };
        write!(f, "{}", error_msg)
    }
}
