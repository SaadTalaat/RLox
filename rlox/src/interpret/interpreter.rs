use super::env::Environment;
use super::error::{RuntimeError, RuntimeErrorKind};
use super::Result;
use crate::callable::{Function, NativeFunction};
use crate::class::Class;
use crate::code::{Code, CodeLocation, HasLocation};
use crate::parse::{Expr, ExprKind, Operator, Stmt, StmtKind};
use crate::LoxValue;
use std::collections::HashMap;
use std::rc::Rc;

pub trait Eval {
    fn eval(&self, interpreter: &mut TreeWalkInterpreter) -> Result<LoxValue>;
}

pub struct TreeWalkInterpreter {
    pub env: Environment,
}

impl TreeWalkInterpreter {
    pub fn new(globals: Vec<NativeFunction>) -> Self {
        let env = Environment::new();
        for f in globals.into_iter() {
            let fn_name = f.name.clone();
            env.define(&fn_name, LoxValue::NF(Rc::new(f)));
        }
        Self { env }
    }

    pub fn run<T: Eval + HasLocation>(&mut self, stmts: Vec<T>, code: &Code) -> Result<()> {
        for stmt in stmts.iter() {
            let result = self.eval(stmt);
            if let Ok(LoxValue::NoValue) = result {
                continue;
            } else if let Err(error) = result {
                // XXX: Move error reporting outside
                println!("{}", "-".repeat(30));
                println!("Error: {}", error);
                code.print_location(&error);
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn eval<T: Eval>(&mut self, expr: &T) -> Result<LoxValue> {
        expr.eval(self)
    }

    pub fn define(&self, key: &str, value: LoxValue) {
        self.env.define(key, value)
    }

    pub fn assign_at(&self, key: &str, value: LoxValue, depth: usize) -> Option<LoxValue> {
        self.env.assign_at(key, value, depth)
    }

    pub fn read_at(&self, key: &str, depth: usize) -> Option<LoxValue> {
        self.env.read_at(key, depth)
    }

    pub fn clone_env(&self) -> Environment {
        self.env.clone()
    }

    pub fn set_env_from_ptr(&mut self, env: &Environment) {
        self.env = env.clone()
    }

    pub fn set_env(&mut self, env: Environment) {
        self.env = env.clone()
    }

    pub fn push_env(&mut self) {
        self.env = self.env.push();
    }

    fn eval_unary<T: Eval + HasLocation>(&mut self, op: &Operator, expr: &T) -> Result<LoxValue> {
        let right: LoxValue = self.eval(expr)?;
        match op {
            Operator::Minus => {
                if let LoxValue::Number(n) = right {
                    Ok(LoxValue::Number(-n))
                } else {
                    Err(RuntimeError::new(
                        RuntimeErrorKind::IllegalUnaryOp,
                        expr.get_location(),
                    ))
                }
            }

            Operator::Bang => Ok(LoxValue::Boolean(!right.is_truthy())),
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::IllegalUnaryOp,
                expr.get_location(),
            )),
        }
    }

    fn eval_binary<T: Eval + HasLocation>(
        &mut self,
        left_expr: &T,
        op: &Operator,
        right_expr: &T,
    ) -> Result<LoxValue> {
        let left: LoxValue = self.eval(left_expr)?;
        let right: LoxValue = self.eval(right_expr)?;

        match op {
            Operator::Minus => Self::subtract(left, right, left_expr.get_location()),
            Operator::Plus => Self::add(left, right, left_expr.get_location()),
            Operator::Slash => Self::division(left, right, left_expr.get_location()),
            Operator::Star => Self::mul(left, right, left_expr.get_location()),
            Operator::Modulo => Self::modulo(left, right, left_expr.get_location()),
            Operator::GreaterThan
            | Operator::GreaterThanEq
            | Operator::LessThan
            | Operator::LessThanEq
            | Operator::EqEq
            | Operator::BangEq => Self::compare(left, op, right, left_expr.get_location()),
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::IllegalBinaryOp,
                left_expr.get_location(),
            )),
        }
    }

    fn eval_ternary<T: Eval>(&mut self, root: &T, left: &T, right: &T) -> Result<LoxValue> {
        if self.eval(root)?.is_truthy() {
            self.eval(left)
        } else {
            self.eval(right)
        }
    }

    fn eval_logical<T: Eval>(&mut self, left: &T, op: &Operator, right: &T) -> Result<LoxValue> {
        let left_val = self.eval(left)?;
        match op {
            Operator::Or if left_val.is_truthy() => Ok(left_val),
            Operator::And if !left_val.is_truthy() => Ok(left_val),
            _ => self.eval(right),
        }
    }

    fn eval_call<T: Eval + HasLocation>(
        &mut self,
        callee_expr: &T,
        arg_exprs: &Vec<T>,
    ) -> Result<LoxValue> {
        let callee = self.eval(callee_expr)?;
        let nargs = arg_exprs.len();
        match callee {
            LoxValue::NF(f) if f.arity != nargs => Err(RuntimeError::new(
                RuntimeErrorKind::MismatchedArgs,
                callee_expr.get_location(),
            )),
            LoxValue::F(f) if f.arity != nargs => Err(RuntimeError::new(
                RuntimeErrorKind::MismatchedArgs,
                callee_expr.get_location(),
            )),
            LoxValue::F(f) => {
                let mut args: Vec<LoxValue> = vec![];
                for arg in arg_exprs.iter() {
                    let result = self.eval(arg)?;
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
                    let result = self.eval(arg)?;
                    args.push(result);
                }
                match f.call(args) {
                    Ok(LoxValue::NoValue) => Ok(LoxValue::Nil),
                    r => r,
                }
            }

            LoxValue::K(class) => {
                let mut args: Vec<LoxValue> = vec![];
                for arg in arg_exprs.iter() {
                    let result = self.eval(arg)?;
                    args.push(result);
                }
                class.call(self, args)
            }
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::NotCallable,
                callee_expr.get_location(),
            )),
        }
    }

    // Helpers
    fn subtract(l_op: LoxValue, r_op: LoxValue, location: &CodeLocation) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l - r)),
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::IllegalBinaryOp,
                location,
            )),
        }
    }

    fn division(l_op: LoxValue, r_op: LoxValue, location: &CodeLocation) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(_), LoxValue::Number(r)) if r == 0.0 => {
                Err(RuntimeError::new(RuntimeErrorKind::ZeroDivision, location))
            }
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l / r)),
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::IllegalBinaryOp,
                location,
            )),
        }
    }

    fn mul(l_op: LoxValue, r_op: LoxValue, location: &CodeLocation) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l * r)),
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::IllegalBinaryOp,
                location,
            )),
        }
    }

    fn modulo(l_op: LoxValue, r_op: LoxValue, location: &CodeLocation) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l % r)),
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::IllegalBinaryOp,
                location,
            )),
        }
    }

    fn add(l_op: LoxValue, r_op: LoxValue, location: &CodeLocation) -> Result<LoxValue> {
        match (l_op, r_op) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l + r)),
            (LoxValue::Str(l), LoxValue::Str(r)) => {
                let new_string = String::from(l.as_str()) + r.as_str();
                Ok(LoxValue::Str(Rc::new(new_string)))
            }
            (LoxValue::Str(l), LoxValue::Number(r)) => {
                let new_string = format!("{}{:.6}", l.as_str(), r);
                Ok(LoxValue::Str(Rc::new(new_string)))
            }
            (LoxValue::Number(l), LoxValue::Str(r)) => {
                let new_string = format!("{:.6}{}", l, r.as_str());
                Ok(LoxValue::Str(Rc::new(new_string)))
            }
            (LoxValue::Str(l), LoxValue::Nil) => {
                let new_string = format!("{}nil", l.as_str());
                Ok(LoxValue::Str(Rc::new(new_string)))
            }
            (LoxValue::Nil, LoxValue::Str(r)) => {
                let new_string = format!("nil{}", r.as_str());
                Ok(LoxValue::Str(Rc::new(new_string)))
            }
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::IllegalBinaryOp,
                location,
            )),
        }
    }

    fn compare(
        l_op: LoxValue,
        op: &Operator,
        r_op: LoxValue,
        location: &CodeLocation,
    ) -> Result<LoxValue> {
        let result = match op {
            Operator::GreaterThan => LoxValue::Boolean(l_op > r_op),
            Operator::GreaterThanEq => LoxValue::Boolean(l_op >= r_op),
            Operator::LessThan => LoxValue::Boolean(l_op < r_op),
            Operator::LessThanEq => LoxValue::Boolean(l_op <= r_op),
            Operator::EqEq => LoxValue::Boolean(l_op == r_op),
            Operator::BangEq => LoxValue::Boolean(l_op != r_op),
            _ => {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::IllegalBinaryOp,
                    location,
                ))
            }
        };
        Ok(result)
    }
}

impl Eval for Stmt {
    fn eval(&self, interpreter: &mut TreeWalkInterpreter) -> Result<LoxValue> {
        match &self.kind {
            StmtKind::Print(expr) => {
                println!("{}", interpreter.eval(expr)?);
                Ok(LoxValue::NoValue)
            }
            StmtKind::Expr(expr) => interpreter.eval(expr),
            StmtKind::Var {
                name,
                init: Some(expr),
            } => {
                let r_value = interpreter.eval(expr)?;
                interpreter.define(name, r_value);
                Ok(LoxValue::NoValue)
            }
            StmtKind::Var { name, .. } => {
                interpreter.define(name, LoxValue::Nil);
                Ok(LoxValue::NoValue)
            }
            StmtKind::Block(stmts) => {
                let tmp_env = interpreter.clone_env();
                interpreter.push_env();
                for stmt in stmts.into_iter() {
                    interpreter.eval(stmt)?;
                }
                interpreter.set_env(tmp_env);
                // TODO: How to return value from function body
                Ok(LoxValue::NoValue)
            }

            StmtKind::If {
                condition,
                then,
                otherwise,
            } => {
                if interpreter.eval(condition)?.is_truthy() {
                    interpreter.eval(then.as_ref())
                } else if let Some(else_block) = otherwise {
                    interpreter.eval(else_block.as_ref())
                } else {
                    Ok(LoxValue::NoValue)
                }
            }

            StmtKind::While { condition, body } => {
                while interpreter.eval(condition)?.is_truthy() {
                    let result = interpreter.eval(body.as_ref());
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
            StmtKind::Function { name, params, body } => {
                let body_rc = Rc::new(*body.clone());
                let func =
                    Function::new(name.to_owned(), params, &body_rc, interpreter.clone_env());
                let func_val = LoxValue::F(Rc::new(func));
                interpreter.define(&name, func_val);
                Ok(LoxValue::NoValue)
            }
            StmtKind::Class {
                name: class_name,
                base,
                methods,
            } => {
                let mut method_list: HashMap<String, Function> = HashMap::new();
                // do we have a base class?
                let maybe_base_cls: Option<LoxValue> = match &base {
                    // If this is a subclass, eval the base class
                    Some(b_expr) => {
                        let base_value = interpreter.eval(b_expr.as_ref())?;
                        match base_value {
                            // make sure the base class is actually a reference
                            // to a class `LoxValue::K`
                            LoxValue::K(_) => Some(base_value),
                            // Cannot inherit from non-class identifiers (e.g. function, variables)
                            // This behavior is handled at resolution time
                            // if it made it to runtime, something is wrong!
                            _ => {
                                return Err(RuntimeError::new(
                                    RuntimeErrorKind::FatalError,
                                    &self.location,
                                ))
                            }
                        }
                    }
                    // Otherwise it's not a base class
                    None => None,
                };
                let old_env = interpreter.clone_env();
                if let Some(base_cls) = &maybe_base_cls {
                    interpreter.push_env();
                    interpreter.define("super", base_cls.clone());
                }

                for method in methods {
                    if let StmtKind::Function { name, params, body } = &method.kind {
                        // Expensive declaration but cheaper binding.
                        let body_rc = Rc::new(*body.clone());
                        let func = Function::new(
                            format!("{}.{}", class_name, name),
                            params,
                            &body_rc,
                            interpreter.clone_env(),
                        );
                        method_list.insert(name.clone(), func);
                    } else {
                        // methods should always resolve to a Function,
                        // otherwise this behavior slipped through
                        // the parser
                        return Err(RuntimeError::new(
                            RuntimeErrorKind::FatalError,
                            &self.location,
                        ));
                    }
                }
                let class = Class::new(class_name, maybe_base_cls.clone(), method_list);
                if maybe_base_cls.is_some() {
                    interpreter.set_env(old_env);
                }
                interpreter.define(&class_name, LoxValue::K(Rc::new(class)));
                Ok(LoxValue::NoValue)
            }
            StmtKind::Return(Some(expr)) => {
                let val = interpreter.eval(expr)?;
                Err(RuntimeError::return_(val))
            }
            StmtKind::Return(None) => Err(RuntimeError::return_(LoxValue::Nil)),
            StmtKind::Break => Err(RuntimeError::break_()),
            StmtKind::Continue => Err(RuntimeError::continue_()),
        }
    }
}

impl Eval for Expr {
    fn eval(&self, interpreter: &mut TreeWalkInterpreter) -> Result<LoxValue> {
        match &self.kind {
            ExprKind::Literal { value } => Ok(value.clone()),
            ExprKind::Grouping { ref expr } => interpreter.eval(expr.as_ref()),
            ExprKind::Unary { operator, expr } => interpreter.eval_unary(operator, expr.as_ref()),

            ExprKind::Binary {
                left,
                operator,
                right,
            } => interpreter.eval_binary(left.as_ref(), operator, right.as_ref()),

            ExprKind::Ternary { root, left, right } => {
                interpreter.eval_ternary(root.as_ref(), left.as_ref(), right.as_ref())
            }

            ExprKind::Logical {
                left,
                operator,
                right,
            } => interpreter.eval_logical(left.as_ref(), operator, right.as_ref()),

            ExprKind::This { depth } => match interpreter.read_at("this", *depth) {
                Some(v) => Ok(v),
                // "this" keyword should always resolve to a value!
                None => Err(RuntimeError::new(
                    RuntimeErrorKind::FatalError,
                    &self.location,
                )),
            },

            ExprKind::Super { property, depth } => {
                let maybe_base_cls = interpreter.read_at("super", *depth);
                // Do we have a base class?
                if let Some(LoxValue::K(base)) = maybe_base_cls {
                    let maybe_instance = interpreter.read_at("this", depth - 1);
                    // Should defintely have a reference to "this"
                    if let Some(LoxValue::I(instance)) = maybe_instance {
                        let maybe_method = base.get_method(property);
                        // does the method exist on the super class?
                        if let Some(method) = maybe_method {
                            Ok(method.bind(&instance))
                        } else {
                            // Method doesn't exist
                            return Err(RuntimeError::new(
                                RuntimeErrorKind::UndefinedProperty,
                                &self.location,
                            ));
                        }
                    } else {
                        // Fatal error, this should exist on the preceding
                        // environment
                        Err(RuntimeError::new(
                            RuntimeErrorKind::FatalError,
                            &self.location,
                        ))
                    }
                } else {
                    // Class has no base class
                    // the behavior should be caught at resolution
                    // time, can't happen here.
                    Err(RuntimeError::new(
                        RuntimeErrorKind::FatalError,
                        &self.location,
                    ))
                }
            }

            ExprKind::Var { name, depth } => match interpreter.read_at(&name, *depth) {
                // Return a copy of the stored value.
                Some(v) => Ok(v),
                None => Err(RuntimeError::new(
                    RuntimeErrorKind::UndeclaredVariable,
                    &self.location,
                )),
            },

            ExprKind::Lambda { params, body } => {
                let body_rc = Rc::new(*body.clone());
                let func = Function::new(
                    "lambda".to_owned(),
                    params,
                    &body_rc,
                    interpreter.clone_env(),
                );
                let lambda = LoxValue::F(Rc::new(func));
                Ok(lambda)
            }

            ExprKind::Assign { name, expr, depth } => {
                let r_value = interpreter.eval(expr.as_ref())?;
                // Return a copy of the assigned value
                interpreter
                    .assign_at(&name, r_value, *depth)
                    .ok_or(RuntimeError::new(
                        RuntimeErrorKind::UndeclaredVariable,
                        &self.location,
                    ))
            }

            ExprKind::Call { callee, args } => interpreter.eval_call(callee.as_ref(), args),

            ExprKind::Get { name, object } => {
                let instance = interpreter.eval(object.as_ref())?;
                match instance {
                    LoxValue::I(instance) => match instance.get(name) {
                        Some(v) => Ok(v),
                        _ => Err(RuntimeError::new(
                            RuntimeErrorKind::UndefinedProperty,
                            &self.location,
                        )),
                    },
                    _ => Err(RuntimeError::new(
                        RuntimeErrorKind::AccessOnPrimitiveType,
                        &self.location,
                    )),
                }
            }

            ExprKind::Set {
                name,
                object,
                value,
            } => {
                let instance = interpreter.eval(object.as_ref())?;
                match instance {
                    LoxValue::I(instance) => {
                        let value = interpreter.eval(value.as_ref())?;
                        instance.set(name, value)
                    }
                    _ => Err(RuntimeError::new(
                        RuntimeErrorKind::AccessOnPrimitiveType,
                        &self.location,
                    )),
                }
            }
        }
    }
}
