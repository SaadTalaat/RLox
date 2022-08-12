use super::env::Environment;
use super::error::RuntimeCtrl;
use super::result::Result;
use crate::lex::{Token, TokenType};
use crate::parse::{Expr, Stmt};
use crate::LiteralValue;

pub struct Interpreter<'a> {
    env: Environment<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    pub fn interpret(&mut self, stmt: &'a Stmt<'a>) -> Result<'a, LiteralValue> {
        match stmt {
            // Expressions e.g.
            // 1 + 2;
            // a + c;
            // true == false;
            Stmt::Expr { expr } => self.evaluate(expr),
            // Variable declarations
            // var name = "value";
            Stmt::Variable { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => LiteralValue::NoValue,
                };
                self.env.define(name, value);
                Ok(LiteralValue::NoValue)
            }
            // Print statements
            // print "value";
            Stmt::Print { expr } => {
                let value = self.evaluate(expr)?;
                println!("{}", value);
                Ok(LiteralValue::NoValue)
            }
            // Code {} blocks
            Stmt::Block { statements } => {
                if statements.len() > 0 {
                    self.env.push_env();
                    for stmt in statements.iter() {
                        self.interpret(stmt)?;
                    }
                    self.env.pop_env()?;
                }
                Ok(LiteralValue::NoValue)
            }
            Stmt::If {
                condition,
                if_body,
                maybe_else_body,
            } => {
                let condition_value = self.evaluate(condition)?;
                if Self::is_truthy(&condition_value)? {
                    self.interpret(if_body)
                } else if let Some(else_body) = maybe_else_body {
                    self.interpret(else_body)
                } else {
                    Ok(LiteralValue::NoValue)
                }
            }
            Stmt::While { condition, body } => {
                while Self::is_truthy(&self.evaluate(condition)?)? {
                    let result = self.interpret(body);
                    if let Err(RuntimeCtrl::BreakEmitted) = result {
                        break;
                    } else if let Err(RuntimeCtrl::ContinueEmitted) = result {
                        continue;
                    }
                }
                Ok(LiteralValue::NoValue)
            }
            Stmt::Break => Err(RuntimeCtrl::BreakEmitted),
            Stmt::Continue => Err(RuntimeCtrl::ContinueEmitted),
        }
    }

    fn evaluate(&mut self, expr: &'a Expr) -> Result<'a, LiteralValue<'a>> {
        match expr {
            Expr::Literal { value } => Ok((*value).clone()),
            // Variable names
            // 1 + a
            Expr::Variable { name } => match self.env.read(name)? {
                LiteralValue::NoValue => Err(RuntimeCtrl::new(
                    name,
                    "Cannot access a variable before it's been initalized".to_owned(),
                )),
                value => Ok(value.clone()),
            },
            // Variable assignment
            // x = 3;
            Expr::Assignment { name, value } => {
                let r_value = self.evaluate(value)?;
                let r_value = self.env.assign(name, r_value)?;
                Ok(r_value.clone())
            }
            // Grouped expression ( 1 + 1 )
            Expr::Grouping { expr } => self.evaluate(expr),
            // Unary expression
            // !true
            // -1
            Expr::Unary { operator, expr } => match operator.token_type {
                // Negate if number, otherwise throw an error
                TokenType::Minus => {
                    let literal = self.evaluate(expr)?;
                    if let LiteralValue::Number(n) = literal {
                        Ok(LiteralValue::Number(-n))
                    } else {
                        // TODO: replace with error
                        Err(RuntimeCtrl::new(
                            operator,
                            "Cannot negate non-numeric literals".to_owned(),
                        ))
                    }
                }
                TokenType::Bang => {
                    let value = self.evaluate(expr)?;
                    let truthy = Self::is_truthy(&value)?;
                    Ok(LiteralValue::Boolean(!truthy))
                }
                _ => Err(RuntimeCtrl::new(
                    operator,
                    format!("Illegal unary operation: {}", operator.token_type),
                )),
            },

            // Binary expression
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left_value = self.evaluate(left)?;
                let right_value = self.evaluate(right)?;

                match operator.token_type {
                    // Subtract
                    TokenType::Minus | TokenType::MinusMinus => {
                        Self::subtract(operator, &left_value, &right_value)
                    }
                    // Multiply
                    TokenType::Star => Self::multiply(operator, &left_value, &right_value),
                    // Division
                    TokenType::Slash => Self::divide(operator, &left_value, &right_value),
                    // Addition
                    // TODO: Don't convert unary --/++ to binary expressions to be able
                    // to return the value after or before the operation, depending on
                    // the operator being a prefix or postfix
                    TokenType::Plus | TokenType::PlusPlus => {
                        Self::add(operator, &left_value, &right_value)
                    }
                    // Modulo
                    TokenType::Modulo => Self::modulo(operator, &left_value, &right_value),
                    // GreaterThan >
                    TokenType::GreaterThan
                    | TokenType::GreaterThanEq
                    | TokenType::LessThan
                    | TokenType::LessThanEq
                    | TokenType::EqEq
                    | TokenType::BangEq => Self::compare(operator, &left_value, &right_value),
                    _ => Err(RuntimeCtrl::new(
                        operator,
                        format!("Illegal binary operation: {}", operator.token_type),
                    )),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left)?;
                let left_is_truthy = Self::is_truthy(&left_value)?;
                match operator.token_type {
                    TokenType::And => {
                        if left_is_truthy {
                            let right_value = self.evaluate(right)?;
                            Ok(right_value)
                        } else {
                            Ok(left_value)
                        }
                    }
                    _ => {
                        if !left_is_truthy {
                            let right_value = self.evaluate(right)?;
                            Ok(right_value)
                        } else {
                            Ok(left_value)
                        }
                    }
                }
            }

            // Ternary expressions
            Expr::Ternary { root, left, right } => {
                let root_value = self.evaluate(root)?;
                self.ternary(&root_value, left, right)
            }
        }
    }

    fn ternary(
        &mut self,
        root: &LiteralValue,
        left: &'a Expr,
        right: &'a Expr,
    ) -> Result<'a, LiteralValue<'a>> {
        // Ternary expressions
        if Self::is_truthy(root)? {
            self.evaluate(left)
        } else {
            self.evaluate(right)
        }
    }

    /// Associated functions.
    fn add<'b>(
        op: &'b Token,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Result<'b, LiteralValue<'b>> {
        match (left, right) {
            (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Number(l + r)),

            (LiteralValue::Str(l), LiteralValue::Str(r)) => {
                Ok(LiteralValue::Str(format!("{}{}", l, r)))
            }

            (LiteralValue::StaticStr(l), LiteralValue::Str(r)) => {
                Ok(LiteralValue::Str(format!("{}{}", l, r)))
            }

            (LiteralValue::Str(l), LiteralValue::StaticStr(r)) => {
                Ok(LiteralValue::Str(format!("{}{}", l, r)))
            }

            (LiteralValue::StaticStr(l), LiteralValue::StaticStr(r)) => {
                Ok(LiteralValue::Str(format!("{}{}", l, r)))
            }

            _ => Err(RuntimeCtrl::new(
                op,
                format!("Cannot add {} to a {}, mismatched types", left, right),
            )),
        }
    }

    fn subtract<'b>(
        op: &'b Token,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Result<'b, LiteralValue<'b>> {
        match (left, right) {
            (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Number(l - r)),
            _ => Err(RuntimeCtrl::new(
                op,
                format!("Cannot subtract a '{}' from a '{}' literal", left, right),
            )),
        }
    }

    fn divide<'b>(
        op: &'b Token,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Result<'b, LiteralValue<'b>> {
        match (left, right) {
            (LiteralValue::Number(l), LiteralValue::Number(r)) if *r != 0.0 => {
                Ok(LiteralValue::Number(l / r))
            }
            (_, LiteralValue::Number(r)) if *r == 0.0 => {
                Err(RuntimeCtrl::new(op, format!("Zero division")))
            }
            _ => Err(RuntimeCtrl::new(
                op,
                format!("Cannot divide {} by {}, mismatched types", left, right),
            )),
        }
    }

    fn multiply<'b>(
        op: &'b Token,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Result<'b, LiteralValue<'b>> {
        match (left, right) {
            (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Number(l * r)),
            _ => Err(RuntimeCtrl::new(
                op,
                format!("Cannot multiply {} by {}, mismatched types", left, right),
            )),
        }
    }

    fn modulo<'b>(
        op: &'b Token,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Result<'b, LiteralValue<'b>> {
        match (left, right) {
            (LiteralValue::Number(l), LiteralValue::Number(r)) if *r != 0.0 => {
                Ok(LiteralValue::Number(l % r))
            }
            (_, LiteralValue::Number(r)) if *r == 0.0 => {
                Err(RuntimeCtrl::new(op, format!("Zero division")))
            }
            _ => Err(RuntimeCtrl::new(
                op,
                format!("Cannot modulo {} by {}, mismatched types", left, right),
            )),
        }
    }

    fn increment<'b>(
        op: &'b Token,
        value: &LiteralValue,
        step: f64,
    ) -> Result<'b, LiteralValue<'b>> {
        match value {
            LiteralValue::Number(l) => Ok(LiteralValue::Number(l + step)),
            _ => Err(RuntimeCtrl::new(
                op,
                format!("Cannot increment/decrement {}, mismatched types", value),
            )),
        }
    }

    fn compare<'b>(
        op: &'b Token,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Result<'b, LiteralValue<'a>> {
        match op.token_type {
            TokenType::GreaterThan => Ok(LiteralValue::Boolean(left > right)),
            TokenType::GreaterThanEq => Ok(LiteralValue::Boolean(left >= right)),
            TokenType::LessThan => Ok(LiteralValue::Boolean(left < right)),
            TokenType::LessThanEq => Ok(LiteralValue::Boolean(left <= right)),
            TokenType::EqEq => Ok(LiteralValue::Boolean(left == right)),
            TokenType::BangEq => Ok(LiteralValue::Boolean(left != right)),
            _ => Err(RuntimeCtrl::new(
                op,
                format!("Cannot compare operands: {} {} {}", left, op, right),
            )),
        }
    }

    fn is_truthy(value: &LiteralValue) -> Result<'a, bool> {
        match *value {
            LiteralValue::Boolean(flag) => Ok(flag),
            LiteralValue::Nil => Ok(false),
            LiteralValue::NoValue => panic!("Illegal value in is_truthy: {}", value),
            _ => Ok(true),
        }
    }
}
