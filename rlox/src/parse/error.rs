use crate::code::CodeLocation;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum ParseErrorKind {
    AlreadyDeclaredIdentifier,
    BaseClassNotAClass,
    // Fatal error, Interpreter Internal error
    FatalError,
    IllegalClassDecl,
    IllegalFunctionDecl,
    IllegalIfStatement,
    IllegalOperator,
    IllegalWhile,
    IllegalFor,
    IllegalVarDeclaration,
    MaximumParamExceeded,
    MissingTernaryColon,
    MissingSemiColon,
    MissingVariableName,
    MissingFunctionName,
    MissingClassName,
    MissingBlockBrace,
    MissingPropertyName,
    NotInALoop,
    NotASubClass,
    ParamExpected,
    // assigning r-value to another r-value
    RvToRvAssignment,
    RecursiveInitializer,
    ReturnAtTopLevel,
    SuperOutsideClass,
    DotExpected,
    TooManyArgs,
    ThisOutsideClass,
    UnbalancedParentheses,
    UnterminatedBlock,
    UnexpectedToken,
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    location: CodeLocation,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, location: &CodeLocation) -> Self {
        Self {
            kind,
            location: location.clone(),
        }
    }
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
