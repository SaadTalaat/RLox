use super::{Result, RuntimeError, RuntimeErrorKind};
use crate::callable::{LoxApplyFn, NativeFunction};
use crate::LoxValue;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Globals {}
impl Globals {
    fn clock(args: Vec<LoxValue>) -> Result<LoxValue> {
        let time = SystemTime::now();
        match time.duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(LoxValue::Number(duration.as_micros() as f64 / 1000.0)),
            Err(_) => Err(RuntimeError::new(RuntimeErrorKind::SystemTimeError)),
        }
    }

    pub fn get() -> Vec<NativeFunction> {
        vec![NativeFunction::new("clock", 0, Self::clock)]
    }
}
