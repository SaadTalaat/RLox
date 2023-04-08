use super::class::Instance;
use super::parse::Stmt;
use super::LoxValue;
use crate::interpret::{Environment, Result, RuntimeErrorKind, TreeWalkInterpreter};
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
        if args.len() != self.arity {
            panic!("Core Failure: native function received wrong number of args.");
        }
        (self.apply)(args)
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    params: Vec<String>,
    body: Box<Stmt>,
    closure: Environment,
}

impl Function {
    pub fn new(name: String, params: Vec<String>, body: Stmt, closure: Environment) -> Self {
        Self {
            name,
            arity: params.len(),
            params,
            body: Box::new(body),
            closure: closure,
        }
    }

    pub fn bind(&self, instance: &Rc<Instance>) -> LoxValue {
        let env = self.closure.push();
        env.define("this", LoxValue::I(instance.clone()));
        LoxValue::F(Self::new(
            self.name.clone(),
            self.params.clone(),
            *self.body.clone(),
            env,
        ))
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
            panic!("Core Failure: function {} received wrong number of args {}, expected {}.", self.name, args.len(), self.arity);
        }
        let globals = interpreter.get_env();
        interpreter.set_env(self.closure.clone());
        interpreter.push_env();
        let zipped = std::iter::zip(self.params.iter(), args.into_iter());
        for (param, arg) in zipped {
            interpreter.define(param, arg);
        }
        let result = interpreter.eval(self.body.as_ref());
        interpreter.set_env(globals);
        match result {
            Err(error) => match error.kind {
                RuntimeErrorKind::RuntimeCtrlReturn(val) => Ok(val),
                _ => Err(error),
            },
            r => r,
        }
    }
}
