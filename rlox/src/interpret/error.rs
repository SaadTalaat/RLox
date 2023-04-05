use crate::LoxValue;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum RuntimeErrorKind {
    IllegalLiteral,
    IllegalUnaryOp,
    IllegalBinaryOperation,
    MismatchedArgs,
    NotCallable,
    NotImplemented,
    RuntimeCtrlReturn(LoxValue),
    RuntimeCtrlBreak,
    RuntimeCtrlContinue,
    SystemTimeError,
    UnrecognizedExpression,
    UndeclaredVariable,
    ZeroDivision,
}

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind) -> Self {
        Self { kind }
    }
}

impl Error for RuntimeError {}
impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
