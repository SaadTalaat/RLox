use crate::code::CodeLocation;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum TokenType {
    // Single character tokens
    Colon,
    Comma,
    Dot,
    LeftParen,
    LeftBrace,
    Minus,
    Modulo,
    Plus,
    Qmark,
    RightParen,
    RightBrace,
    SemiColon,
    Slash,
    Star,

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
    Number,
    String,

    // Keywords
    And,
    Break,
    Class,
    Continue,
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
