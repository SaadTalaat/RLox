use super::error::{RuntimeError, RuntimeErrorKind};
use super::Result;
use crate::LoxValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct EnvElement {
    map: HashMap<String, LoxValue>,
    pub parent: Option<Rc<RefCell<EnvElement>>>,
}

impl EnvElement {
    pub fn new(parent: Option<Rc<RefCell<Self>>>) -> Self {
        Self {
            map: HashMap::new(),
            parent,
        }
    }

    pub fn define(&mut self, key: &str, value: LoxValue) {
        self.map.insert(key.to_owned(), value);
    }

    pub fn read(&self, key: &str) -> Result<LoxValue> {
        let maybe_val = self.map.get(key);
        if let Some(val) = maybe_val {
            Ok(val.clone())
        } else if let Some(ref p) = self.parent {
            p.borrow().read(key)
        } else {
            Err(RuntimeError::new(RuntimeErrorKind::UndeclaredVariable))
        }
    }

    pub fn assign(&mut self, key: &str, value: LoxValue) -> Result<LoxValue> {
        if self.map.contains_key(key) {
            self.map.insert(key.to_owned(), value);
            Ok(self.map.get(key).unwrap().clone())
        } else if let Some(ref p) = self.parent {
            p.borrow_mut().assign(key, value)
        } else {
            Err(RuntimeError::new(RuntimeErrorKind::UndeclaredVariable))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    elem: Rc<RefCell<EnvElement>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            elem: Rc::new(RefCell::new(EnvElement::new(None))),
        }
    }

    pub fn push(&self) -> Self {
        let elem = EnvElement::new(Some(self.elem.clone()));
        Environment {
            elem: Rc::new(RefCell::new(elem)),
        }
    }

    pub fn read(&self, key: &str) -> Result<LoxValue> {
        self.elem.borrow().read(key)
    }

    pub fn assign(&self, key: &str, value: LoxValue) -> Result<LoxValue> {
        self.elem.borrow_mut().assign(key, value)
    }

    pub fn define(&self, key: &str, value: LoxValue) {
        self.elem.borrow_mut().define(key, value)
    }
}
