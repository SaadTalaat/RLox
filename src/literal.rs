use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LiteralValue<'a> {
    NoValue,
    Nil,
    Number(f64),
    StaticStr(&'a str),
    Str(String),
    Boolean(bool),
}

impl<'a> Display for LiteralValue<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LiteralValue::Number(num) => write!(f, "{}", num),
            LiteralValue::Str(str_ref) => write!(f, "{}", &str_ref),
            LiteralValue::StaticStr(str_ref) => write!(f, "{}", &str_ref),
            LiteralValue::Nil => write!(f, "nil"),
            LiteralValue::Boolean(b) => write!(f, "{}", b),
            LiteralValue::NoValue => Err(fmt::Error),
        }
    }
}
