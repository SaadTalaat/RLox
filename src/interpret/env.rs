use super::Result;
use super::RuntimeCtrl;
use crate::lex::Token;
use crate::LiteralValue;
use std::collections::HashMap;

/// Environment<'a>
/// Given 'a denotes the lifetime of the source
/// code 'str'.
pub struct Environment<'a> {
    maps: Vec<HashMap<&'a str, LiteralValue<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            maps: vec![HashMap::new()],
        }
    }

    /// Overwrites key with value and returns the old value, or creates a new entry.
    pub fn define(&mut self, key: &'a Token, value: LiteralValue<'a>) {
        let last_idx = self.maps.len() - 1;
        self.maps[last_idx].insert(key.lexeme(), value);
    }

    pub fn read(&self, token: &'a Token) -> Result<'a, &LiteralValue<'a>> {
        let key = token.lexeme();
        for map in self.maps.iter().rev() {
            let value = map.get(key);
            if let Some(v) = value {
                return Ok(v);
            }
        }
        Err(RuntimeCtrl::new(
            token,
            format!("Undeclared variable {}", key),
        ))
    }

    pub fn assign(
        &mut self,
        token: &'a Token,
        value: LiteralValue<'a>,
    ) -> Result<'a, &LiteralValue<'a>> {
        let key = token.lexeme();
        let mut idx: i32 = self.maps.len() as i32 - 1;
        let last_idx = idx as usize;
        while idx >= 0 {
            // Just a cast to index into maps
            let cursor = idx as usize;
            if self.maps[cursor].contains_key(key) {
                self.maps[cursor].insert(key, value);
                return Ok(self.maps[cursor].get(key).unwrap());
            }
            idx -= 1;
        }

        Err(RuntimeCtrl::new(
            token,
            format!("Undeclared variable {}", key),
        ))
    }

    pub fn push_env(&mut self) {
        self.maps.push(HashMap::new());
    }

    pub fn pop_env(&mut self) -> Result<'a, ()> {
        if self.maps.len() > 1 {
            self.maps.pop();
            Ok(())
        } else {
            panic!("Internal error, attempting to delete global scope")
        }
    }
}
