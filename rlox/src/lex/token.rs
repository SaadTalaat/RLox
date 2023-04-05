use crate::code::CodeLocation;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Modulo,
    Colon,
    SemiColon,
    Slash,
    Star,
    Qmark,

    // One or two character tokens
    Bang,
    BangEq,
    Equal,
    EqEq,
    GreaterThan,
    GreaterThanEq,
    LessThan,
    LessThanEq,
    MinusMinus,
    PlusPlus,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Break,
    Continue,

    // EOF
    EOF,
}

impl Display for TokenType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub location: CodeLocation,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        cursor: usize,
        line: usize,
        line_offset: usize,
        length: usize,
    ) -> Self {
        Self {
            token_type,
            location: CodeLocation::new(cursor, line, line_offset, length),
        }
    }
}

impl Display for Token {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.token_type)
    }
}
