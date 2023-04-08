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

    pub fn read_at(&self, key: &str, depth: i32) -> Option<LoxValue> {
        if depth > 0 {
            if let Some(p) = &self.parent {
                p.borrow().read_at(key, depth - 1)
            } else {
                None
            }
        } else if let Some(val) = self.map.get(key) {
            Some(val.clone())
        } else {
            None
        }
    }

    pub fn assign_at(&mut self, key: &str, value: LoxValue, depth: i32) -> Option<LoxValue> {
        if depth > 0 {
            if let Some(p) = &self.parent {
                p.borrow_mut().assign_at(key, value, depth - 1)
            } else {
                None
            }
        } else if self.map.contains_key(key) {
            self.map.insert(key.to_owned(), value);
            Some(self.map.get(key).unwrap().clone())
        } else {
            None
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

    pub fn read_at(&self, key: &str, depth: usize) -> Option<LoxValue> {
        self.elem.borrow().read_at(key, depth as i32)
    }

    pub fn assign_at(&self, key: &str, value: LoxValue, depth: usize) -> Option<LoxValue> {
        self.elem.borrow_mut().assign_at(key, value, depth as i32)
    }

    pub fn define(&self, key: &str, value: LoxValue) {
        self.elem.borrow_mut().define(key, value)
    }
}
