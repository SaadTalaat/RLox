use super::lex::{Token, TokenType};
use super::LoxValue;
#[derive(Debug, Clone, Copy)]
pub struct CodeLocation {
    // global cursor to the starting character of the location
    cursor: usize,
    // line number
    line: usize,
    // Offset local to the line
    line_offset: usize,
    // length of the location
    length: usize,
}

impl CodeLocation {
    pub fn new(cursor: usize, line: usize, line_offset: usize, length: usize) -> Self {
        Self {
            cursor,
            line,
            line_offset,
            length,
        }
    }
}

pub struct Code<'a> {
    source: &'a [u8],
}

impl<'a> Code<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
        }
    }

    pub fn lexeme(&self, location: CodeLocation) -> &str {
        let lexeme_utf8 = &self.source[location.cursor..(location.cursor + location.length)];
        // XXX: Unexpected behavior
        std::str::from_utf8(lexeme_utf8).unwrap()
    }

    pub fn get_identifier(&self, token: &Token) -> String {
        match token.token_type {
            TokenType::Identifier => {
                self.lexeme(token.location).to_owned()
            }
            _ => panic!("Cannot extract name for identifier {}", token),
        }
    }
    pub fn get_value(&self, token: &Token) -> LoxValue {
        match token.token_type {
            TokenType::Nil => LoxValue::Nil,
            TokenType::True => LoxValue::Boolean(true),
            TokenType::False => LoxValue::Boolean(false),
            TokenType::Number => {
                // panic at this point if token-type is number but
                // the lexeme doesn't represent a valid number.
                LoxValue::Number(self.lexeme(token.location).parse::<f64>().unwrap())
            }
            TokenType::String => {
                // Lose the quotations.
                let location = CodeLocation {
                    cursor: token.location.cursor + 1,
                    length: token.location.length - 2,
                    ..token.location
                };
                LoxValue::Str(self.lexeme(location).to_owned())
            }
            _ => LoxValue::NoValue,
        }
    }
}
