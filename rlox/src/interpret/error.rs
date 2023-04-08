use crate::code::{CodeLocation, HasLocation};
use crate::LoxValue;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum RuntimeErrorKind {
    FatalError,
    AccessOnPrimitiveType,
    IllegalInheritance,
    IllegalLiteral,
    IllegalUnaryOp,
    IllegalBinaryOperation,
    MismatchedArgs,
    NotCallable,
    NotImplemented,
    NoBaseClass,
    RuntimeCtrlReturn(LoxValue),
    RuntimeCtrlBreak,
    RuntimeCtrlContinue,
    SystemTimeError,
    UnrecognizedExpression,
    UndeclaredVariable,
    UndefinedProperty,
    ZeroDivision,
}

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub location: Option<CodeLocation>,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind, location: &CodeLocation) -> Self {
        Self {
            kind,
            location: Some(location.clone()),
        }
    }

    pub fn return_(value: LoxValue) -> Self {
        Self {
            kind: RuntimeErrorKind::RuntimeCtrlReturn(value),
            location: None,
        }
    }

    pub fn break_() -> Self {
        Self {
            kind: RuntimeErrorKind::RuntimeCtrlBreak,
            location: None,
        }
    }

    pub fn continue_() -> Self {
        Self {
            kind: RuntimeErrorKind::RuntimeCtrlContinue,
            location: None,
        }
    }
}

impl Error for RuntimeError {}
impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.kind)
    }
}

impl HasLocation for RuntimeError {
    fn get_location(&self) -> &CodeLocation {
        match &self.location {
            Some(loc) => loc,
            None => panic!("Fatal error!"),
        }
    }
}
