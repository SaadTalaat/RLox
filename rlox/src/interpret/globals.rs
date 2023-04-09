use super::{Result, RuntimeError, RuntimeErrorKind};
use crate::callable::{NativeFunction};
use crate::LoxValue;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Globals {}
impl Globals {
    fn clock(_: Vec<LoxValue>) -> Result<LoxValue> {
        let time = SystemTime::now();
        match time.duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(LoxValue::Number(duration.as_micros() as f64 / 1000.0)),
            // XXX: Find a way to pass location to errors here.
            Err(_) => panic!("Time went backwads?!"),
        }
    }

    fn exit(args: Vec<LoxValue>) -> Result<LoxValue> {
        // at this point, arity should be well verified.
        let errno_value = args.get(0).unwrap();
        match errno_value {
            LoxValue::Number(errno) => std::process::exit(*errno as i32),
            // XXX: Find a way to pass location to errors here.
            _ => std::process::exit(128),
        }
    }

    pub fn get() -> Vec<NativeFunction> {
        vec![
            NativeFunction::new("clock", 0, Self::clock),
            NativeFunction::new("exit", 1, Self::exit),
        ]
    }
}
