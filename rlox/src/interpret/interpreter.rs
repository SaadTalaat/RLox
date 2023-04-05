use super::env::Environment;
use super::error::{RuntimeError, RuntimeErrorKind};
use super::globals::Globals;
use super::Result;
use crate::callable::Function;
use crate::parse::{Expr, Operator, Stmt};
use crate::LoxValue;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TreeWalkInterpreter {
    pub env: Vec<Rc<RefCell<Environment>>>,
}

impl TreeWalkInterpreter {
    pub fn new() -> Self {
        let globals = Globals::get();
        let mut env = Environment::new();
        for f in globals.into_iter() {
            env.define(&f.name.clone(), LoxValue::NF(f));
        }
        Self {
            env: vec![Rc::new(RefCell::new(env))],
        }
    }
    pub fn run(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts.into_iter() {
            let result = self.interpret(&stmt);
            if let Ok(LoxValue::NoValue) = result {
                continue;
            }
            result.unwrap();
        }
    }

    pub fn push(&mut self, env: &Rc<RefCell<Environment>>) {
        self.env.push(Rc::clone(env));
    }

    pub fn pop(&mut self) {
        self.env.pop();
    }

    fn env(&self) -> Rc<RefCell<Environment>> {
        let last = self.env.get(self.env.len() - 1).unwrap();
        Rc::clone(last)
    }

    pub fn interpret(&mut self, stmt: &Stmt) -> Result<LoxValue> {
        match stmt {
            Stmt::Print(expr) => {
                println!("{}", self.evaluate(expr)?);
                Ok(LoxValue::NoValue)
            }
            Stmt::Expr(expr) => self.evaluate(expr),
            Stmt::Var {
                name,
                init: Some(expr),
            } => {
                let r_value = self.evaluate(expr)?;
                self.env().borrow_mut().define(name, r_value);
                Ok(LoxValue::NoValue)
            }
            Stmt::Var { name, .. } => {
                self.env().borrow_mut().define(name, LoxValue::Nil);
                Ok(LoxValue::NoValue)
            }
            Stmt::Block(stmts) => {
                self.env().borrow_mut().push();
                let mut result = LoxValue::NoValue;
                for stmt in stmts.into_iter() {
                    result = self.interpret(stmt)?;
                }
                self.env().borrow_mut().pop();
                // TODO: How to return value from function body
                Ok(LoxValue::NoValue)
            }

            Stmt::If {
                condition,
                then,
                otherwise,
            } => {
                if Self::is_truthy(&self.evaluate(condition)?) {
                    self.interpret(then)
                } else if let Some(else_block) = otherwise {
                    self.interpret(else_block)
                } else {
                    Ok(LoxValue::NoValue)
                }
            }

            Stmt::While { condition, body } => {
                while Self::is_truthy(&self.evaluate(condition)?) {
                    let result = self.interpret(body);
                    match result {
                        Err(err) => match err.kind {
                            RuntimeErrorKind::RuntimeCtrlBreak => break,
                            RuntimeErrorKind::RuntimeCtrlContinue => continue,
                            _ => return Err(err),
                        },
                        _ => (),
                    }
                }
                Ok(LoxValue::NoValue)
            }
            Stmt::Function { name, params, body } => {
                let func = LoxValue::F(Function::new(
                    name.to_owned(),
                    params.clone(),
                    *body.clone(),
                    &self.env(),
                ));
                self.env().borrow_mut().define(&name, func);
                Ok(LoxValue::NoValue)
            }
            Stmt::Return(Some(expr)) => {
                let val = self.evaluate(&expr)?;
                Err(RuntimeError::new(RuntimeErrorKind::RuntimeCtrlReturn(val)))
            }
            Stmt::Return(None) => Err(RuntimeError::new(RuntimeErrorKind::RuntimeCtrlReturn(
                LoxValue::Nil,
            ))),
            Stmt::Break => Err(RuntimeError::new(RuntimeErrorKind::RuntimeCtrlBreak)),
            Stmt::Continue => Err(RuntimeError::new(RuntimeErrorKind::RuntimeCtrlContinue)),
        }
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<LoxValue> {
        match expression {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Grouping { expr } => self.evaluate(expr),
            Expr::Unary { operator, expr } => self.evaluate_unary(operator, expr),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right),
            Expr::Ternary { root, left, right } => self.evaluate_ternary(root, left, right),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.evaluate_logical(left, operator, right),
            Expr::Var { name } => match self.env().borrow().read(&name) {
                // Return a copy of the stored value.
                Ok(v) => Ok(v.clone()),
                Err(e) => Err(e),
            },
            Expr::Assign { name, expr } => {
                let r_value = self.evaluate(expr)?;
                // Return a copy of the assigned value
                Ok(self.env().borrow_mut().assign(name, r_value)?.clone())
            }
            Expr::Call { callee, args } => self.evaluate_call(callee, args),
            _ => Err(RuntimeError::new(RuntimeErrorKind::UnrecognizedExpression)),
        }
    }

    fn evaluate_unary(&mut self, op: &Operator, expr: &Expr) -> Result<LoxValue> {
        let right: LoxValue = self.evaluate(expr)?;
        match op {
            Operator::Minus => {
                if let LoxValue::Number(n) = right {
                    Ok(LoxValue::Number(-n))
                } else {
                    Err(RuntimeError::new(RuntimeErrorKind::IllegalUnaryOp))
                }
            }

            Operator::Bang => Ok(LoxValue::Boolean(!Self::is_truthy(&right))),
            _ => Err(RuntimeError::new(RuntimeErrorKind::IllegalUnaryOp)),
        }
    }

    fn evaluate_binary(&mut self, left: &Expr, op: &Operator, right: &Expr) -> Result<LoxValue> {
        let left: LoxValue = self.evaluate(left)?;
        let right: LoxValue = self.evaluate(right)?;

        match op {
            Operator::Minus => Self::subtract(left, right),
            Operator::Plus => Self::add(left, right),
            Operator::Slash => Self::division(left, right),
            Operator::Star => Self::mul(left, right),
            Operator::Modulo => Self::modulo(left, right),
            Operator::GreaterThan
            | Operator::GreaterThanEq
            | Operator::LessThan
            | Operator::LessThanEq
            | Operator::EqEq
            | Operator::BangEq => Self::compare(left, op, right),
            _ => Err(RuntimeError::new(RuntimeErrorKind::IllegalBinaryOperation)),
        }
    }

    fn evaluate_ternary(&mut self, root: &Expr, left: &Expr, right: &Expr) -> Result<LoxValue> {
        if Self::is_truthy(&self.evaluate(root)?) {
            self.evaluate(left)
        } else {
            self.evaluate(right)
        }
    }

    fn evaluate_logical(&mut self, left: &Expr, op: &Operator, right: &Expr) -> Result<LoxValue> {
        let left_val = self.evaluate(left)?;
        match op {
            Operator::Or if Self::is_truthy(&left_val) => Ok(left_val),
            Operator::And if !Self::is_truthy(&left_val) => Ok(left_val),
            _ => self.evaluate(right),
        }
    }

    fn evaluate_call(&mut self, callee_expr: &Expr, arg_exprs: &Vec<Expr>) -> Result<LoxValue> {
        let callee = self.evaluate(callee_expr)?;
        let nargs = arg_exprs.len();
        match callee {
            LoxValue::NF(f) if f.arity != nargs => {
                Err(RuntimeError::new(RuntimeErrorKind::MismatchedArgs))
            }
            LoxValue::F(f) if f.arity != nargs => {
                Err(RuntimeError::new(RuntimeErrorKind::MismatchedArgs))
            }
            LoxValue::F(f) => {
                let mut args: Vec<LoxValue> = vec![];
                for arg in arg_exprs.iter() {
                    let result = self.evaluate(arg)?;
                    args.push(result);
                }
                match f.call(self, args) {
                    Ok(LoxValue::NoValue) => Ok(LoxValue::Nil),
                    r => r,
                }
            }
            LoxValue::NF(f) => {
                let mut args: Vec<LoxValue> = vec![];
                for arg in arg_exprs.iter() {
                    let result = self.evaluate(arg)?;
                    args.push(result);
                }
                match f.call(args) {
                    Ok(LoxValue::NoValue) => Ok(LoxValue::Nil),
                    r => r,
                }
            }
            _ => Err(RuntimeError::new(RuntimeErrorKind::NotCallable)),
        }
    }

    // Helpers
    fn is_truthy(value: &LoxValue) -> bool {
        match value {
            LoxValue::Boolean(flag) => *flag,
            LoxValue::Nil => false,
            LoxValue::NoValue => panic!("Illegal value in is_truthy"),
            _ => true,
        }
    }

    fn subtract(l_op: LoxValue, r_op: LoxValue) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l - r)),
            _ => Err(RuntimeError::new(RuntimeErrorKind::IllegalBinaryOperation)),
        }
    }

    fn division(l_op: LoxValue, r_op: LoxValue) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(_), LoxValue::Number(r)) if r == 0.0 => {
                Err(RuntimeError::new(RuntimeErrorKind::ZeroDivision))
            }
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l / r)),
            _ => Err(RuntimeError::new(RuntimeErrorKind::IllegalBinaryOperation)),
        }
    }

    fn mul(l_op: LoxValue, r_op: LoxValue) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l * r)),
            _ => Err(RuntimeError::new(RuntimeErrorKind::IllegalBinaryOperation)),
        }
    }

    fn modulo(l_op: LoxValue, r_op: LoxValue) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l % r)),
            _ => Err(RuntimeError::new(RuntimeErrorKind::IllegalBinaryOperation)),
        }
    }

    fn add(l_op: LoxValue, r_op: LoxValue) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l + r)),
            (LoxValue::Str(l), LoxValue::Str(r)) => Ok(LoxValue::Str(l + r.as_str())),
            (LoxValue::Str(l), LoxValue::Number(r)) => Ok(LoxValue::Str(format!("{}{}", l, r))),
            (LoxValue::Number(l), LoxValue::Str(r)) => Ok(LoxValue::Str(format!("{}{}", l, r))),
            _ => Err(RuntimeError::new(RuntimeErrorKind::IllegalBinaryOperation)),
        }
    }

    fn compare(l_op: LoxValue, op: &Operator, r_op: LoxValue) -> Result<LoxValue> {
        let result = match op {
            Operator::GreaterThan => LoxValue::Boolean(l_op > r_op),
            Operator::GreaterThanEq => LoxValue::Boolean(l_op >= r_op),
            Operator::LessThan => LoxValue::Boolean(l_op < r_op),
            Operator::LessThanEq => LoxValue::Boolean(l_op <= r_op),
            Operator::EqEq => LoxValue::Boolean(l_op == r_op),
            Operator::BangEq => LoxValue::Boolean(l_op != r_op),
            _ => return Err(RuntimeError::new(RuntimeErrorKind::IllegalBinaryOperation)),
        };
        Ok(result)
    }
}
