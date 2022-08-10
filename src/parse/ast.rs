use crate::lex::Token;
use crate::LiteralValue;
use std::fmt::{self, Display, Formatter};

pub trait ExprHandler<T> {
    fn handle(&self, expr: &Expr) -> T;
}

#[derive(Debug)]
pub enum Expr<'a> {
    Literal {
        value: &'a LiteralValue<'a>,
    },
    Variable {
        name: &'a Token<'a>,
    },
    Assignment {
        name: &'a Token<'a>,
        value: Box<Expr<'a>>,
    },
    Ternary {
        root: Box<Expr<'a>>,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>,
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

    pub fn variable(token: &'a Token<'a>) -> Expr<'a> {
        Expr::Variable { name: token }
    }

    pub fn ternary(root: Expr<'a>, left: Expr<'a>, right: Expr<'a>) -> Expr<'a> {
        Expr::Ternary {
            root: Box::new(root),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn binary(left: Expr<'a>, operator: &'a Token, right: Expr<'a>) -> Expr<'a> {
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

    pub fn assignment(name: &'a Token, value: Expr<'a>) -> Expr<'a> {
        Expr::Assignment {
            name: name,
            value: Box::new(value),
        }
    }
}

impl<'a> Display for Expr<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expr::Ternary { root, left, right } => {
                write!(f, "({} ? {} : {})", root, left, right)
            }
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
            Expr::Variable { name } => {
                write!(f, "{}", name)
            }
            Expr::Assignment { name, value } => {
                write!(f, "{} = {}", name, value)
            }
        }
    }
}

pub enum Stmt<'a> {
    Expr {
        expr: Expr<'a>,
    },
    Print {
        expr: Expr<'a>,
    },
    Variable {
        name: &'a Token<'a>,
        initializer: Option<Expr<'a>>,
    },
    Block {
        statements: Vec<Stmt<'a>>,
    },
}

impl<'a> Stmt<'a> {
    pub fn expr(expr: Expr<'a>) -> Self {
        Self::Expr { expr }
    }

    pub fn print(expr: Expr<'a>) -> Self {
        Self::Print { expr }
    }

    pub fn variable(name: &'a Token, initializer: Option<Expr<'a>>) -> Self {
        Self::Variable { name, initializer }
    }

    pub fn block(statements: Vec<Stmt<'a>>) -> Self {
        Self::Block { statements }
    }
}

impl<'a> Display for Stmt<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Stmt::Expr { expr } => write!(f, "{}", expr),
            Stmt::Print { expr } => write!(f, "print {}", expr),
            Stmt::Variable {
                name,
                initializer: Some(initializer),
            } => write!(f, "var {} = {}", name, initializer),
            Stmt::Variable {
                name,
                initializer: None,
            } => write!(f, "var {}", name),
            Stmt::Block { statements } => {
                write!(f, "{{ block (statements {}) }}", statements.len())
            }
        }
    }
}
