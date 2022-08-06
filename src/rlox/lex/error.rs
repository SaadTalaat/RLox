use std::error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Error<'a> {
    line_index: usize,
    offset: usize,
    illegal_literal: char,
    source: &'a str,
    message: &'a str,
}

impl<'a> Error<'a> {
    pub fn new(
        line_index: usize,
        offset: usize,
        illegal_literal: char,
        source: &'a [u8],
        message: &'a str,
    ) -> Self {
        Error {
            line_index,
            offset,
            illegal_literal,
            source: std::str::from_utf8(source).unwrap(),
            message,
        }
    }
}

impl<'a> error::Error for Error<'a> {}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // First, write the offending line to the output.
        let lines: Vec<&str> = self.source.lines().collect();
        let line = lines[self.line_index];
        let line_prefix = format!("line({}): ", self.line_index + 1);
        write!(f, "{}{}\n", line_prefix, line)?;
        // Then, write a cursor pointing to the offending literal
        let error_msg = {
            let indent_size = self.offset + line_prefix.len() + 1;
            let indentation: Vec<u8> = (0..indent_size).map(|_| b' ').collect();
            let indentation = std::str::from_utf8(&indentation).unwrap();
            format!(
                "{}^ {}: {}",
                indentation, self.illegal_literal, self.message
            )
        };
        write!(f, "{}", error_msg)
    }
}
