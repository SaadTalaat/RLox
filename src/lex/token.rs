use std::fmt::{self, Display, Formatter};
use std::str;

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

    // Literals
    Identifier,
    Number,
    String,

    // Keywords
    While,
    For,
    If,
    Else,
    True,
    False,
    Nil,
    Or,
    And,
    Print,
    Return,
    Super,
    This,
    Var,
    Class,
    // Function Body function(param: int) => { body }
    EqGreaterThan,

    // EOF
    EOF,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Token
#[derive(Debug)]
pub struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    line: usize,
    offset: usize,
    file_offset: usize,
}

impl<'a> Token<'a> {
    pub fn new(
        token_type: TokenType,
        lexeme: &'a [u8],
        line: usize,
        offset: usize,
        file_offset: usize,
    ) -> Self {
        Token {
            token_type,
            lexeme: std::str::from_utf8(lexeme).unwrap(),
            line,
            offset,
            file_offset,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn lexeme(&self) -> &'a str {
        self.lexeme
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Token[type=({}), lexeme=({})",
            self.token_type,
            // Should panic, lexme should be always
            // a string
            self.lexeme
        )
    }
}
