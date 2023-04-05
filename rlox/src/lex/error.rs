use crate::code::CodeLocation;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum LexicalErrorKind {
    UnrecognizedLiterl,
    IllegalIdentifer,
    UnbalancedBlockComment,
    UnterminatedString,
}

#[derive(Debug)]
pub struct LexicalError {
    kind: LexicalErrorKind,
    location: CodeLocation,
}

impl LexicalError {
    pub fn new(kind: LexicalErrorKind, cursor: usize, line: usize, line_offset: usize) -> Self {
        Self {
            kind,
            location: CodeLocation::new(cursor, line, line_offset, 1),
        }
    }
}

impl Error for LexicalError {}

impl Display for LexicalError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
