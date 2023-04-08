use super::callable::Function;
use super::value::LoxValue;
use crate::interpret::{Result, TreeWalkInterpreter};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub arity: usize,
    base_class: Option<Rc<Class>>,
    methods: HashMap<String, Rc<Function>>,
}

impl Class {
    pub fn new(name: &str, base: Option<LoxValue>, methods: HashMap<String, Function>) -> Self {
        // does it have a base class?
        let base_class: Option<Rc<Self>> = match base {
            Some(value) => match value {
                LoxValue::K(b) => Some(b),
                // Should be always LoxValue::K at this point
                _ => panic!("Class::new called with base class of illegal value"),
            },
            _ => None,
        };

        // Determine arity
        let mut cls_arity: usize = 0;
        let init = methods.get("init");
        if let Some(Function { arity, .. }) = init {
            cls_arity = *arity;
        } else if let Some(base) = &base_class {
            cls_arity = base.arity;
        }

        Self {
            name: name.to_owned(),
            arity: cls_arity,
            base_class,
            methods: methods.into_iter().map(|(k, v)| (k, Rc::new(v))).collect(),
        }
    }

    pub fn call(
        self: &Rc<Self>,
        interpreter: &mut TreeWalkInterpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue> {
        if args.len() != self.arity {
            panic!(
                "Core Failure: constructor {} received wrong number of args {}, expected {}.",
                self.name,
                args.len(),
                self.arity
            );
        }

        let instance = Rc::new(Instance::new(self));
        if let Some(initializer) = self.get_method("init") {
            let bound_init = initializer.bind(&instance);

            if let LoxValue::F(inner_init) = bound_init {
                inner_init.call(interpreter, args)?;
            }
        }
        Ok(LoxValue::I(instance))
    }

    pub fn get_method(&self, name: &str) -> Option<&Rc<Function>> {
        match self.methods.get(name) {
            None => match &self.base_class {
                Some(b) => b.get_method(name),
                None => None,
            },
            result => result,
        }
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        // two classes never equal each others
        // unless..
        // XXX: raise runtime error?
        false
    }
}

impl PartialOrd for Class {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // two classes are incomparable,
        // unless
        // XXX: raise runtime error?
        None
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub class: Rc<Class>,
    pub properties: Rc<RefCell<HashMap<String, LoxValue>>>,
}

impl Instance {
    pub fn new(class: &Rc<Class>) -> Self {
        Self {
            class: class.clone(),
            properties: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(self: &Rc<Self>, name: &str) -> Option<LoxValue> {
        match self.properties.borrow().get(name) {
            None => {
                let method = self.class.get_method(name)?;
                Some(method.bind(self))
            }
            Some(v) => Some(v.clone()),
        }
    }

    pub fn set(&self, name: &str, value: LoxValue) -> Result<LoxValue> {
        self.properties.borrow_mut().insert(name.to_owned(), value);
        Ok(LoxValue::NoValue)
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        // two instances never equal each others
        // unless..
        // XXX: a cmp function is provided by the
        // instance.
        false
    }
}

impl PartialOrd for Instance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // two instances are incomparable,
        // unless
        // XXX: a cmp function is provided by the
        // instance
        None
    }
}

impl Display for Instance {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "<instance {}>", self.class.name)
    }
}
