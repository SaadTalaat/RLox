use super::error::{ParseError, ParseErrorKind};
use super::Result;
use crate::lex::{Token, TokenType};
use crate::LoxValue;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Operator {
    Minus,
    Plus,
    Slash,
    Star,
    Modulo,
    Bang,
    BangEq,
    EqEq,
    GreaterThan,
    GreaterThanEq,
    LessThan,
    LessThanEq,
    And,
    Or,
    Function(String),
}

impl Operator {
    pub fn from_token(token: &Token, lexeme: &str) -> Result<Self> {
        let op = match token.token_type {
            TokenType::Minus => Self::Minus,
            TokenType::Plus => Self::Plus,
            TokenType::Slash => Self::Slash,
            TokenType::Star => Self::Star,
            TokenType::Modulo => Self::Modulo,
            TokenType::EqEq => Self::EqEq,
            TokenType::Bang => Self::Bang,
            TokenType::BangEq => Self::BangEq,
            TokenType::GreaterThan => Self::GreaterThan,
            TokenType::GreaterThanEq => Self::GreaterThanEq,
            TokenType::LessThan => Self::LessThan,
            TokenType::LessThanEq => Self::LessThanEq,
            TokenType::And => Self::And,
            TokenType::Or => Self::Or,
            TokenType::Identifier => Self::Function(lexeme.to_owned()),
            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::IllegalOperator,
                    token.location,
                ))
            }
        };
        Ok(op)
    }
}

impl Display for Operator {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let repr = match self {
            Self::Minus => "-",
            Self::Plus => "+",
            Self::Slash => "/",
            Self::Star => "*",
            Self::Modulo => "%",
            Self::EqEq => "==",
            Self::Bang => "!",
            Self::BangEq => "!=",
            Self::GreaterThan => ">",
            Self::GreaterThanEq => ">=",
            Self::LessThan => "<",
            Self::LessThanEq => "<=",
            Self::And => "and",
            Self::Or => "or",
            Self::Function(name) => name,
        };
        write!(formatter, "{}", repr)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Block(Vec<Stmt>),
    Var {
        name: String,
        init: Option<Expr>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
    },
    If {
        condition: Expr,
        then: Box<Stmt>,
        otherwise: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
}

impl Stmt {
    pub fn variable(name: String, init: Option<Expr>) -> Self {
        Self::Var { name, init }
    }

    pub fn if_stmt(condition: Expr, then: Self, otherwise: Option<Self>) -> Self {
        Self::If {
            condition,
            then: Box::new(then),
            otherwise: otherwise.map(|body| Box::new(body)),
        }
    }

    pub fn while_stmt(condition: Expr, body: Self) -> Self {
        Self::While {
            condition,
            body: Box::new(body),
        }
    }

    pub fn function(name: String, params: Vec<String>, body: Self) -> Self {
        Self::Function {
            name,
            params,
            body: Box::new(body),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let repr = match self {
            Stmt::Expr(expr) => format!("{};", expr),
            Stmt::Print(expr) => format!("print {};", expr),
            Stmt::Var {
                name,
                init: Some(val),
            } => format!("var {} = {};", name, val),
            Stmt::Var { name, init: None } => format!("var {};", name),
            Stmt::Function { name, params, .. } => format!("fun {name} ({} params)", params.len()),
            Stmt::Block(stmts) => format!("{{ block (statements {}) }}", stmts.len()),
            Stmt::If {
                condition,
                then,
                otherwise: None,
            } => format!("if ({condition}) {then}"),
            Stmt::If {
                condition,
                then,
                otherwise: Some(stmt),
            } => format!("if ({condition}) {then} else {stmt}"),
            Stmt::While { condition, body } => format!("while ({condition}) {body}"),
            Stmt::Return(Some(val)) => format!("return {val};"),
            Stmt::Return(None) => format!("return;"),
            Stmt::Break => format!("break"),
            Stmt::Continue => format!("continue"),
        };
        write!(formatter, "{}", repr)
    }
}

// TODO: Implement Something like the visitor pattern using generics
// i.e.
// impl Eval for Grouping {
//      ...
// }
//
// What this needs is breaking the Expr Enum types into
// enum ExprKind {
//   ..
// }
// struct Grouping {
//      type: ExprKind::Grouping,
//      ...
// }
//
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expr {
    Literal {
        value: LoxValue,
    },
    Binary {
        left: Box<Expr>,
        operator: Operator,
        right: Box<Expr>,
    },
    Unary {
        operator: Operator,
        expr: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Ternary {
        root: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Operator,
        right: Box<Expr>,
    },
    Var {
        name: String,
        depth: usize,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Stmt>,
    },
    Assign {
        name: String,
        expr: Box<Expr>,
        depth: usize,
    },
}

impl Expr {
    pub fn literal(value: LoxValue) -> Self {
        Self::Literal { value }
    }

    pub fn ternary(root: Self, left: Self, right: Self) -> Self {
        Self::Ternary {
            root: Box::new(root),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn binary(left: Self, operator: Operator, right: Self) -> Self {
        Self::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn logical(left: Self, operator: Operator, right: Self) -> Self {
        Self::Logical {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn unary(operator: Operator, expr: Self) -> Self {
        Self::Unary {
            operator,
            expr: Box::new(expr),
        }
    }

    pub fn call_expr(callee: Self, args: Vec<Self>) -> Self {
        Self::Call {
            callee: Box::new(callee),
            args,
        }
    }

    pub fn grouping(expr: Self) -> Self {
        Self::Grouping {
            expr: Box::new(expr),
        }
    }

    pub fn variable(name: String) -> Self {
        Self::Var { name, depth: 0 }
    }

    pub fn lambda(params: Vec<String>, body: Stmt) -> Self {
        Self::Lambda {
            params,
            body: Box::new(body),
        }
    }
    pub fn assign(name: String, expr: Expr) -> Self {
        Self::Assign {
            name,
            expr: Box::new(expr),
            depth: 0,
        }
    }

    pub fn set_depth(&mut self, new_depth: usize) {
        match self {
            Self::Var { depth, .. } | Self::Assign { depth, .. } => {
                *depth = new_depth;
            }
            _ => panic!("cannot set depth to non variable referencing expr"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Literal { value } => write!(formatter, "{}", value),
            Self::Ternary { root, left, right } => {
                write!(formatter, "({} ? {} : {})", root, left, right)
            }
            Self::Binary {
                left,
                operator,
                right,
            } => write!(formatter, "({} {} {})", operator, left, right),
            Self::Unary { operator, expr } => write!(formatter, "({} {})", operator, expr),
            Self::Logical {
                left,
                operator,
                right,
            } => write!(formatter, "({} {} {})", operator, left, right),
            Self::Grouping { expr } => write!(formatter, "(group {})", expr),
            Self::Var { name, .. } => write!(formatter, "{}", name),
            Self::Lambda { params, .. } => write!(formatter, "<lambda ({} params)", params.len()),
            Self::Assign { name, expr, .. } => write!(formatter, "{} = {}", name, expr),
            Self::Call { callee, args } => write!(formatter, "{}(nargs {})", callee, args.len()),
        }
    }
}
