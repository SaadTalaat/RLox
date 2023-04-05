use super::error::{RuntimeError, RuntimeErrorKind};
use super::Result;
use crate::LoxValue;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    maps: Vec<HashMap<String, LoxValue>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            maps: vec![HashMap::new()],
        }
    }

    pub fn from_vec(maps: Vec<HashMap<String, LoxValue>>) -> Self {
        Self { maps }
    }

    pub fn push(&mut self) {
        self.maps.push(HashMap::new())
    }

    pub fn pop(&mut self) {
        self.maps.pop();
    }

    pub fn define(&mut self, key: &str, value: LoxValue) {
        let top_env_index = self.maps.len() - 1;
        self.maps[top_env_index].insert(key.to_owned(), value);
    }

    pub fn read(&self, key: &str) -> Result<&LoxValue> {
        for map in self.maps.iter().rev() {
            if let Some(val) = map.get(key) {
                return Ok(val);
            }
        }
        Err(RuntimeError::new(RuntimeErrorKind::UndeclaredVariable))
    }

    pub fn assign(&mut self, key: &str, value: LoxValue) -> Result<&LoxValue> {
        let mut index: i32 = self.maps.len() as i32 - 1;

        while index >= 0 {
            let uindex = index as usize;
            if self.maps[uindex].contains_key(key) {
                self.maps[uindex].insert(key.to_owned(), value);
                let value = self.maps[uindex].get(key).unwrap();
                return Ok(&value);
            }
            index -= 1;
        }
        Err(RuntimeError::new(RuntimeErrorKind::UndeclaredVariable))
    }
}
