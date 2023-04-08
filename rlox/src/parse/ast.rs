use super::error::{ParseError, ParseErrorKind};
use super::Result;
use crate::code::{CodeLocation, HasLocation};
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
                    &token.location,
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
pub enum StmtKind {
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
    Class {
        name: String,
        base: Option<Box<Expr>>,
        methods: Vec<Stmt>,
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Stmt {
    pub kind: StmtKind,
    pub location: CodeLocation,
}

impl Stmt {
    pub fn new(kind: StmtKind, location: CodeLocation) -> Self {
        Self { kind, location }
    }

    pub fn variable(name: String, init: Option<Expr>, location: CodeLocation) -> Self {
        let kind = StmtKind::Var { name, init };
        Self::new(kind, location)
    }

    pub fn if_stmt(
        condition: Expr,
        then: Self,
        otherwise: Option<Self>,
        location: CodeLocation,
    ) -> Self {
        let kind = StmtKind::If {
            condition,
            then: Box::new(then),
            otherwise: otherwise.map(|body| Box::new(body)),
        };
        Self::new(kind, location)
    }

    pub fn while_stmt(condition: Expr, body: Self, location: CodeLocation) -> Self {
        let kind = StmtKind::While {
            condition,
            body: Box::new(body),
        };
        Self::new(kind, location)
    }

    pub fn function(name: String, params: Vec<String>, body: Self, location: CodeLocation) -> Self {
        let kind = StmtKind::Function {
            name,
            params,
            body: Box::new(body),
        };
        Self::new(kind, location)
    }

    pub fn class(
        name: String,
        base: Option<Expr>,
        methods: Vec<Self>,
        location: CodeLocation,
    ) -> Self {
        let kind = StmtKind::Class {
            name,
            methods,
            base: base.map(|b| Box::new(b)),
        };
        Self::new(kind, location)
    }

    pub fn block(stmts: Vec<Self>, location: CodeLocation) -> Self {
        let kind = StmtKind::Block(stmts);
        Self::new(kind, location)
    }

    pub fn expr(expr: Expr) -> Self {
        let location = expr.location;
        let kind = StmtKind::Expr(expr);
        Self::new(kind, location)
    }

    pub fn print(expr: Expr, location: CodeLocation) -> Self {
        let kind = StmtKind::Print(expr);
        Self::new(kind, location)
    }

    pub fn return_(maybe_expr: Option<Expr>, location: CodeLocation) -> Self {
        let kind = StmtKind::Return(maybe_expr);
        Self::new(kind, location)
    }

    pub fn break_(location: CodeLocation) -> Self {
        let kind = StmtKind::Break;
        Self::new(kind, location)
    }
    pub fn continue_(location: CodeLocation) -> Self {
        let kind = StmtKind::Continue;
        Self::new(kind, location)
    }
}

impl Display for Stmt {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let repr = match &self.kind {
            StmtKind::Expr(expr) => format!("{};", expr),
            StmtKind::Print(expr) => format!("print {};", expr),
            StmtKind::Var {
                name,
                init: Some(val),
            } => format!("var {} = {};", name, val),
            StmtKind::Var { name, init: None } => format!("var {};", name),
            StmtKind::Function { name, params, .. } => {
                format!("fun {name} ({} params)", params.len())
            }
            StmtKind::Class { name, methods, .. } => {
                format!("class {name} {{ <{} methods> }}", methods.len())
            }
            StmtKind::Block(stmts) => format!("{{ block (statements {}) }}", stmts.len()),
            StmtKind::If {
                condition,
                then,
                otherwise: None,
            } => format!("if ({condition}) {then}"),
            StmtKind::If {
                condition,
                then,
                otherwise: Some(stmt),
            } => format!("if ({condition}) {then} else {stmt}"),
            StmtKind::While { condition, body } => format!("while ({condition}) {body}"),
            StmtKind::Return(Some(val)) => format!("return {val};"),
            StmtKind::Return(None) => format!("return;"),
            StmtKind::Break => format!("break"),
            StmtKind::Continue => format!("continue"),
        };
        write!(formatter, "{}", repr)
    }
}

impl HasLocation for Stmt {
    fn get_location(&self) -> &CodeLocation {
        &self.location
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ExprKind {
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
    Get {
        name: String,
        object: Box<Expr>,
    },
    Set {
        name: String,
        object: Box<Expr>,
        value: Box<Expr>,
    },
    This {
        depth: usize,
    },
    Super {
        property: String,
        depth: usize,
    },
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Expr {
    pub kind: ExprKind,
    pub location: CodeLocation,
}

impl Expr {
    pub fn new(kind: ExprKind, location: CodeLocation) -> Self {
        Self { kind, location }
    }

    pub fn literal(value: LoxValue, location: CodeLocation) -> Self {
        Self::new(ExprKind::Literal { value }, location)
    }

    pub fn ternary(root: Self, left: Self, right: Self, location: CodeLocation) -> Self {
        let kind = ExprKind::Ternary {
            root: Box::new(root),
            left: Box::new(left),
            right: Box::new(right),
        };
        Self::new(kind, location)
    }

    pub fn binary(left: Self, operator: Operator, right: Self, location: CodeLocation) -> Self {
        let kind = ExprKind::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        };
        Self::new(kind, location)
    }

    pub fn logical(left: Self, operator: Operator, right: Self, location: CodeLocation) -> Self {
        let kind = ExprKind::Logical {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        };
        Self::new(kind, location)
    }

    pub fn unary(operator: Operator, expr: Self, location: CodeLocation) -> Self {
        let kind = ExprKind::Unary {
            operator,
            expr: Box::new(expr),
        };
        Self::new(kind, location)
    }

    pub fn call_expr(callee: Self, args: Vec<Self>, location: CodeLocation) -> Self {
        let kind = ExprKind::Call {
            callee: Box::new(callee),
            args,
        };
        Self::new(kind, location)
    }

    pub fn grouping(expr: Self, location: CodeLocation) -> Self {
        let kind = ExprKind::Grouping {
            expr: Box::new(expr),
        };
        Self::new(kind, location)
    }

    pub fn variable(name: String, location: CodeLocation) -> Self {
        let kind = ExprKind::Var { name, depth: 0 };
        Self::new(kind, location)
    }

    pub fn lambda(params: Vec<String>, body: Stmt, location: CodeLocation) -> Self {
        let kind = ExprKind::Lambda {
            params,
            body: Box::new(body),
        };
        Self::new(kind, location)
    }
    pub fn assign(name: String, expr: Self, location: CodeLocation) -> Self {
        let kind = ExprKind::Assign {
            name,
            expr: Box::new(expr),
            depth: 0,
        };
        Self::new(kind, location)
    }

    pub fn get(name: String, object: Self, location: CodeLocation) -> Self {
        let kind = ExprKind::Get {
            name,
            object: Box::new(object),
        };
        Expr::new(kind, location)
    }

    pub fn set(name: String, object: Self, value: Self, location: CodeLocation) -> Self {
        let kind = ExprKind::Set {
            name,
            object: Box::new(object),
            value: Box::new(value),
        };
        Expr::new(kind, location)
    }

    pub fn this(location: CodeLocation) -> Self {
        Expr::new(ExprKind::This { depth: 0 }, location)
    }

    pub fn super_(property: String, location: CodeLocation) -> Self {
        Expr::new(ExprKind::Super { property, depth: 0 }, location)
    }

    pub fn set_depth(&mut self, new_depth: usize) {
        match &mut self.kind {
            ExprKind::Var { depth, .. }
            | ExprKind::Assign { depth, .. }
            | ExprKind::This { depth }
            | ExprKind::Super { depth, .. } => {
                *depth = new_depth;
            }
            _ => panic!("cannot set depth to non variable referencing expr"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.kind {
            ExprKind::Literal { value } => write!(formatter, "{}", value),
            ExprKind::Ternary { root, left, right } => {
                write!(formatter, "({} ? {} : {})", root, left, right)
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => write!(formatter, "({} {} {})", operator, left, right),
            ExprKind::Unary { operator, expr } => write!(formatter, "({} {})", operator, expr),
            ExprKind::Logical {
                left,
                operator,
                right,
            } => write!(formatter, "({} {} {})", operator, left, right),
            ExprKind::Grouping { expr } => write!(formatter, "(group {})", expr),
            ExprKind::Var { name, .. } => write!(formatter, "{}", name),
            ExprKind::Lambda { params, .. } => {
                write!(formatter, "<lambda ({} params)", params.len())
            }
            ExprKind::Assign { name, expr, .. } => write!(formatter, "{} = {}", name, expr),
            ExprKind::Call { callee, args } => {
                write!(formatter, "{}(nargs {})", callee, args.len())
            }
            ExprKind::Get { name, object } => write!(formatter, "{}.{}", object, name),
            ExprKind::Set {
                name,
                object,
                value,
            } => write!(formatter, "{}.{} = {}", object, name, value),
            ExprKind::This { .. } => write!(formatter, "this"),
            ExprKind::Super { .. } => write!(formatter, "super"),
        }
    }
}

impl HasLocation for Expr {
    fn get_location(&self) -> &CodeLocation {
        &self.location
    }
}
