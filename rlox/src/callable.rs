use super::parse::Stmt;
use super::LoxValue;
use crate::interpret::{Environment, Result, RuntimeErrorKind, TreeWalkInterpreter};
use std::cell::RefCell;
use std::rc::Rc;

pub type LoxApplyFn = fn(Vec<LoxValue>) -> Result<LoxValue>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    apply: LoxApplyFn,
}

impl NativeFunction {
    pub fn new(name: &str, arity: usize, apply: LoxApplyFn) -> Self {
        Self {
            name: name.to_owned(),
            arity,
            apply,
        }
    }
    pub fn call(&self, args: Vec<LoxValue>) -> Result<LoxValue> {
        (self.apply)(args)
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    params: Vec<String>,
    body: Box<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<String>,
        body: Stmt,
        closure: &Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name,
            arity: params.len(),
            params,
            body: Box::new(body),
            closure: Rc::clone(closure),
        }
    }
    pub fn call(
        &self,
        interpreter: &mut TreeWalkInterpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue> {
        // It's interpreter's job to make sure we have the correct
        // number of arguments, so panic in case we have wrong number
        // of arguments.
        if args.len() != self.arity {
            panic!("Core Failure: Function received wrong number of args.");
        }
        self.closure.borrow_mut().push();
        let zipped = std::iter::zip(self.params.iter(), args.into_iter());
        for (param, arg) in zipped {
            self.closure.borrow_mut().define(param, arg);
        }
        interpreter.push(&self.closure);
        let result = interpreter.interpret(&*self.body);
        interpreter.env.pop();
        self.closure.borrow_mut().pop();
        match result {
            Err(error) => match error.kind {
                RuntimeErrorKind::RuntimeCtrlReturn(val) => Ok(val),
                _ => Err(error),
            },
            r => r,
        }
    }
}
