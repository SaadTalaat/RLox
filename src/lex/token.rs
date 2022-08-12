use crate::LiteralValue;
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
    Modulo,
    Qmark,
    Colon,

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
    Break,
    Continue,
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
    pub token_type: TokenType,
    pub value: LiteralValue<'a>,
    lexeme: &'a str,
    pub line_str: &'a str,
    pub line_index: usize,
    pub offset: usize,
    file_offset: usize,
}

impl<'a> Token<'a> {
    pub fn new(
        token_type: TokenType,
        lexeme: &'a [u8],
        line_str: &'a str,
        line_index: usize,
        offset: usize,
        file_offset: usize,
    ) -> Self {
        let lexeme = std::str::from_utf8(lexeme).unwrap();
        let value = Token::value(&token_type, lexeme);
        Token {
            token_type,
            lexeme,
            line_str,
            line_index,
            offset,
            file_offset,
            value,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn lexeme(&self) -> &'a str {
        self.lexeme
    }

    // token type reference should only live for the
    // duration of the function call, or more.
    // `lexeme` and `LiteralValue`, should live for the
    // duration of `Token`
    fn value<'b>(token_type: &TokenType, lexeme: &'a str) -> LiteralValue<'a> {
        match token_type {
            TokenType::False => LiteralValue::Boolean(false),
            TokenType::True => LiteralValue::Boolean(true),
            TokenType::Nil => LiteralValue::Nil,
            TokenType::Number => {
                // panic, at this stage we shouldn't have non digits in
                // number tokens.
                LiteralValue::Number(lexeme.parse::<f64>().unwrap())
            }
            TokenType::String => {
                // Remove the quotes.
                LiteralValue::StaticStr(&lexeme[1..lexeme.len() - 1])
            }
            _ => LiteralValue::NoValue,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}
