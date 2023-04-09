use super::code::{Code, HasLocation};
use std::fmt::Display;

pub struct ErrorJournal<'a> {
    code: &'a Code<'a>,
}

impl<'a> ErrorJournal<'a> {
    pub fn new(code: &'a Code) -> Self {
        Self { code }
    }

    pub fn report<T: HasLocation + Display>(&self, error: &T) {
        eprintln!("---- Error ----");
        eprintln!("{}", error);
        self.code.print_location(error);
    }
}
