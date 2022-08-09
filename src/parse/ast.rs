use crate::lex::{LiteralValue, Token};
use std::fmt::{self, Display, Formatter};

pub trait ExprHandler<T> {
    fn handle(&self, expr: &Expr) -> T;
}

pub enum Expr<'a> {
    Literal {
        value: &'a LiteralValue<'a>,
    },
    Binary {
        left: Box<Expr<'a>>,
        operator: &'a Token<'a>,
        right: Box<Expr<'a>>,
    },
    Grouping {
        expr: Box<Expr<'a>>,
    },
    Unary {
        operator: &'a Token<'a>,
        expr: Box<Expr<'a>>,
    },
}

impl<'a> Expr<'a> {
    pub fn accept<T>(&self, handler: &impl ExprHandler<T>) -> T {
        handler.handle(self)
    }

    pub fn literal(value: &'a LiteralValue) -> Expr<'a> {
        Expr::Literal { value }
    }

    pub fn binary(
        left: Expr<'a>,
        operator: &'a Token,
        right: Expr<'a>,
    ) -> Expr<'a> {
        Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn grouping(expr: Expr<'a>) -> Expr<'a> {
        Expr::Grouping {
            expr: Box::new(expr),
        }
    }

    pub fn unary(operator: &'a Token, expr: Expr<'a>) -> Expr<'a> {
        Expr::Unary {
            operator,
            expr: Box::new(expr),
        }
    }
}

impl<'a> Display for Expr<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator, left, right)
            }
            Expr::Literal { value } => write!(f, "{}", value),
            Expr::Grouping { expr } => write!(f, "(group {})", expr),
            Expr::Unary { operator, expr } => {
                write!(f, "({} {})", operator, expr)
            }
        }
    }
}
