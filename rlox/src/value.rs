use super::callable::{Function, NativeFunction};
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub enum LoxValue {
    NoValue,
    Nil,
    Number(f64),
    Boolean(bool),
    Str(String),
    NF(NativeFunction),
    F(Function),
}

impl PartialEq for LoxValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NoValue, _) | (_, Self::NoValue) => panic!("Cannot compare `NoValue`"),
            (Self::Nil, Self::Nil) => true,
            // Numbers
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::Number(_), _) | (_, Self::Number(_)) => false,
            // Booleans
            (Self::Boolean(l), Self::Boolean(r)) => l == r,
            (Self::Boolean(_), _) | (_, Self::Boolean(_)) => false,
            // Str
            (Self::Str(l), Self::Str(r)) => l == r,
            (Self::Str(_), _) | (_, Self::Str(_)) => false,
            // NF
            // native functions are global functions
            // without namespaces..so we're good.
            (Self::NF(l), Self::NF(r)) => l.name == r.name,
            (Self::NF(_), _) | (_, Self::NF(_)) => false,
            // F
            // XXX: Don't compare functiions?
            _ => false,
        }
    }
}
impl PartialOrd for LoxValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::NoValue, _) | (_, Self::NoValue) => panic!("Cannot compare `NoValue`"),
            (Self::NF(_), _) | (_, Self::NF(_)) | (Self::F(_), _) | (_, Self::F(_)) => {
                panic!("Cannot compare functions")
            }
            (Self::Nil, Self::Nil) => Some(Ordering::Equal),
            (Self::Nil, _) => Some(Ordering::Less),
            (_, Self::Nil) => Some(Ordering::Greater),
            // Booleans
            (Self::Boolean(l), Self::Boolean(r)) => l.partial_cmp(r),
            (Self::Boolean(l), Self::Number(r)) => (if *l { &1.0 } else { &0.0 }).partial_cmp(r),
            (Self::Number(l), Self::Boolean(r)) => l.partial_cmp(if *r { &1.0 } else { &0.0 }),
            (Self::Boolean(_), _) => Some(Ordering::Less),
            (_, Self::Boolean(_)) => Some(Ordering::Greater),
            // Numbers
            (Self::Number(l), Self::Number(r)) => l.partial_cmp(r),
            (Self::Number(_), _) => Some(Ordering::Less),
            (_, Self::Number(_)) => Some(Ordering::Greater),
            // Str
            (Self::Str(l), Self::Str(r)) => l.partial_cmp(r),
            (Self::Str(_), _) => Some(Ordering::Less),
            (_, Self::Str(_)) => Some(Ordering::Greater),
        }
    }
}

impl Display for LoxValue {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let repr = match self {
            Self::NoValue => return Err(fmt::Error),
            Self::Nil => "nil".to_owned(),
            Self::Number(num) => format!("{}", num),
            Self::Boolean(b) => format!("{}", b),
            Self::Str(s) => format!("{}", s),
            Self::NF(f) => format!("function({})", f.name),
            Self::F(f) => format!("function({})", f.name),
        };
        write!(formatter, "{}", repr)
    }
}
