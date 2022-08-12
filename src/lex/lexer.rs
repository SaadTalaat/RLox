use super::error::LexicalError;
use super::result::Result;
use super::token::{Token, TokenType};

pub struct Lexer<'a> {
    // Points to the character being scanned
    cursor: usize,
    // The index of the current line we're on.
    line: usize,
    // The cursor offset local to the line.
    line_offset: usize,
    // keeps record of source code size to
    // avoid calling source.len() during
    // look-aheads
    source_size: usize,
    // The source code in bytes.
    source: &'a [u8],
    // The source code seperated to lines,
    // this is really handy when creating
    // informative errors.
    lines: Vec<&'a str>,
    // If the lexer has emitted the EOF
    // Token. It's marked as closed.
    closed: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: 0,
            line: 0,
            line_offset: 0,
            source: source.as_bytes(),
            lines: source.lines().collect(),
            source_size: source.len(),
            closed: false,
        }
    }

    // Determines whether the lexeme is an
    // identifier or a reserved keyword.
    fn identifier_type(&self, lexeme: &'a [u8]) -> Result<'a, TokenType> {
        let lexeme: &str = std::str::from_utf8(lexeme).map_err(|_| {
            LexicalError::new(
                self.line,
                self.line_offset,
                lexeme[0] as char,
                lexeme,
                "Illegal identifier or keyword",
            )
        })?;

        let token_type = match lexeme {
            "or" => TokenType::Or,
            "and" => TokenType::And,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "nil" => TokenType::Nil,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "var" => TokenType::Var,
            "class" => TokenType::Class,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            _ => TokenType::Identifier,
        };
        return Ok(token_type);
    }
    // Returns a character ahead of the cursor given
    // Example:
    // source = "test"
    // cursor = 0
    // result = 'e'
    fn look_ahead(&self) -> char {
        if self.cursor < (self.source_size - 1) {
            self.source[self.cursor + 1] as char
        } else {
            '\0'
        }
    }

    fn at_end(&self) -> bool {
        self.cursor >= self.source_size
    }

    fn current_line(&self) -> &'a str {
        self.lines[self.line]
    }

    fn advance(&mut self) {
        self.cursor += 1;
        self.line_offset += 1;
    }

    fn newline(&mut self) {
        self.cursor += 1;
        self.line_offset = 0;
        self.line += 1;
    }

    fn scan_comment(&mut self) {
        // Skip initial 2 //
        self.advance();
        self.advance();
        while !self.at_end() {
            if self.source[self.cursor] == b'\n' {
                self.newline();
                break;
            }
            self.advance();
        }
    }

    fn scan_block_comment(&mut self) -> Result<'a, ()> {
        let start_line = self.line;
        let start_line_offset = self.line_offset;
        let start_cursor = self.cursor;
        // Skip initial /
        self.advance();
        let mut block_count = 1;
        while !self.at_end() {
            let chr = self.source[self.cursor] as char;
            if chr == '*' && self.look_ahead() == '/' {
                block_count -= 1;
                self.advance();
                self.advance();
            } else if chr == '/' && self.look_ahead() == '*' {
                block_count += 1;

                self.advance();
                self.advance();
            } else if chr == '\n' {
                self.newline();
            } else {
                self.advance();
            }

            if block_count == 0 {
                break;
            }
        }

        if block_count != 0 {
            Err(LexicalError::new(
                start_line,
                start_line_offset,
                self.source[start_cursor] as char,
                self.source.clone(),
                "Unbalanced block comment",
            ))
        } else {
            Ok(())
        }
    }

    fn get_string(&mut self) -> Result<'a, Token<'a>> {
        let start_line = self.line;
        let start_line_offset = self.line_offset;
        let start_cursor = self.cursor;
        // Skip initial "
        self.advance();
        let mut terminated = false;
        while !self.at_end() {
            let chr = self.source[self.cursor] as char;
            if chr == '"' {
                self.advance();
                terminated = true;
                break;
            } else {
                self.advance();
            }
        }

        if !terminated {
            Err(LexicalError::new(
                start_line,
                start_line_offset,
                self.source[start_cursor] as char,
                self.source,
                "Unterminated string literal",
            ))
        } else {
            let lexeme = &self.source[start_cursor..self.cursor];
            let token = Token::new(
                TokenType::String,
                lexeme,
                self.current_line(),
                start_line,
                start_line_offset,
                start_cursor,
            );
            Ok(token)
        }
    }

    fn get_number(&mut self) -> Result<'a, Token<'a>> {
        let start_line = self.line;
        let start_line_offset = self.line_offset;
        let start_cursor = self.cursor;

        let mut dot_consumed = false;
        self.advance();
        while !self.at_end() {
            let chr = self.source[self.cursor] as char;
            match chr {
                '0'..='9' => self.advance(),
                '.' if !dot_consumed => {
                    dot_consumed = true;
                    match self.look_ahead() {
                        '0'..='9' => {
                            self.advance();
                            self.advance()
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }

        let lexeme = &self.source[start_cursor..self.cursor];
        let token = Token::new(
            TokenType::Number,
            lexeme,
            self.current_line(),
            start_line,
            start_line_offset,
            start_cursor,
        );
        Ok(token)
    }

    fn scan_identifier(&mut self) -> Result<'a, Token<'a>> {
        let start_line = self.line;
        let start_line_offset = self.line_offset;
        let start_cursor = self.cursor;
        // Consume initial digit
        self.advance();
        while !self.at_end() {
            let chr = self.source[self.cursor] as char;
            match chr {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => self.cursor += 1,
                _ => break,
            }
        }

        let lexeme = &self.source[start_cursor..self.cursor];
        let token_type = self.identifier_type(lexeme)?;
        let token = Token::new(
            token_type,
            lexeme,
            self.current_line(),
            start_line,
            start_line_offset,
            start_cursor,
        );
        Ok(token)
    }

    fn get_token(&mut self, ttype: TokenType, len: usize) -> Token<'a> {
        let lexeme = &self.source[self.cursor..self.cursor + 1];
        let token = Token::new(
            ttype,
            lexeme,
            self.current_line(),
            self.line,
            self.line_offset,
            self.cursor,
        );
        for _ in 0..len {
            self.advance();
        }
        token
    }

    fn scan_next(&mut self) -> Result<'a, Token<'a>> {
        if self.at_end() {
            return Ok(Token::new(
                TokenType::EOF,
                b"\0",
                self.lines.last().unwrap(),
                self.line - 1,
                self.lines.last().unwrap().len(),
                self.cursor,
            ));
        }
        let chr = self.source[self.cursor] as char;

        let token = match chr {
            ' ' | '\r' | '\t' => {
                self.advance();
                self.scan_next()
            }
            '\n' => {
                self.newline();
                self.scan_next()
            }
            '(' => Ok(self.get_token(TokenType::LeftParen, 1)),
            ')' => Ok(self.get_token(TokenType::RightParen, 1)),
            '{' => Ok(self.get_token(TokenType::LeftBrace, 1)),
            '}' => Ok(self.get_token(TokenType::RightBrace, 1)),
            ',' => Ok(self.get_token(TokenType::Comma, 1)),
            '.' => Ok(self.get_token(TokenType::Dot, 1)),
            '-' => match self.look_ahead() {
                '-' => Ok(self.get_token(TokenType::MinusMinus, 2)),
                _ => Ok(self.get_token(TokenType::Minus, 1)),
            },
            '+' => match self.look_ahead() {
                '+' => Ok(self.get_token(TokenType::PlusPlus, 2)),
                _ => Ok(self.get_token(TokenType::Plus, 1)),
            },
            ';' => Ok(self.get_token(TokenType::SemiColon, 1)),
            '*' => Ok(self.get_token(TokenType::Star, 1)),
            '%' => Ok(self.get_token(TokenType::Modulo, 1)),
            '?' => Ok(self.get_token(TokenType::Qmark, 1)),
            ':' => Ok(self.get_token(TokenType::Colon, 1)),

            '=' => match self.look_ahead() {
                '=' => Ok(self.get_token(TokenType::EqEq, 2)),
                '>' => Ok(self.get_token(TokenType::EqGreaterThan, 2)),
                _ => Ok(self.get_token(TokenType::Equal, 1)),
            },

            '!' => match self.look_ahead() {
                '=' => Ok(self.get_token(TokenType::BangEq, 2)),
                _ => Ok(self.get_token(TokenType::Bang, 1)),
            },

            '>' => match self.look_ahead() {
                '=' => Ok(self.get_token(TokenType::GreaterThanEq, 2)),
                _ => Ok(self.get_token(TokenType::GreaterThan, 1)),
            },

            '<' => match self.look_ahead() {
                '=' => Ok(self.get_token(TokenType::LessThanEq, 2)),
                _ => Ok(self.get_token(TokenType::LessThan, 1)),
            },

            '/' if self.look_ahead() == '/' => {
                self.scan_comment();
                self.scan_next()
            }

            '/' if self.look_ahead() == '*' => {
                self.scan_block_comment();
                self.scan_next()
            }

            '/' => Ok(self.get_token(TokenType::Slash, 1)),
            '"' => self.get_string(),
            '0'..='9' => self.get_number(),
            '_' | 'a'..='z' | 'A'..='Z' => self.scan_identifier(),
            _ => {
                return Err(LexicalError::new(
                    self.line,
                    self.line_offset.clone(),
                    chr,
                    self.source,
                    "Unrecognized literal",
                ))
            }
        };
        token
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<'a, Token<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.scan_next() {
            Ok(token) => match token.token_type {
                TokenType::EOF if !self.closed => {
                    self.closed = true;
                    Some(Ok(token))
                }
                TokenType::EOF if self.closed => None,
                _ => Some(Ok(token)),
            },
            err => Some(err),
        }
    }
}
