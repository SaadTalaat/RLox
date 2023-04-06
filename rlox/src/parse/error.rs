use crate::code::CodeLocation;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum ParseErrorKind {
    IllegalFunctionDecl,
    IllegalIfStatement,
    IllegalOperator,
    IllegalWhile,
    IllegalFor,
    MaximumParamExceeded,
    MissingTernaryColon,
    MissingSemiColon,
    MissingVariableName,
    MissingFunctionName,
    MissingBlockBrace,
    ParamExpected,
    // assigning r-value to another r-value
    RvToRvAssignment,
    TooManyArgs,
    UnbalancedParentheses,
    UnterminatedBlock,
    UnexpectedToken,
    // Fatal error
    FatalError,
    RecursiveInitializer,
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    location: CodeLocation,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, location: CodeLocation) -> Self {
        Self {
            kind,
            location: location,
        }
    }
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
