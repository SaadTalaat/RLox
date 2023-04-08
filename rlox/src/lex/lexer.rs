use super::error::{LexicalError, LexicalErrorKind};
use super::token::{Token, TokenType};
use super::Result;

pub struct Lexer<'a> {
    // Points to the character being scanned.
    cursor: usize,
    // Index to the current line we're scanning.
    line: usize,
    // Cursor local to the line we're scanning.
    line_offset: usize,
    // Source code.
    source: &'a [u8],
    // Source code size in bytes.
    source_size: usize,
    // If Lexer has emitted EOF Token, it's marked as closed.
    closed: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: 0,
            line: 1,
            line_offset: 0,
            source: source.as_bytes(),
            source_size: source.len(),
            closed: false,
        }
    }

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

    fn step(&mut self, count: usize) {
        self.cursor += count;
        self.line_offset += count;
    }

    fn new_line(&mut self) {
        self.cursor += 1;
        self.line += 1;
        // reset line offset
        self.line_offset = 0;
    }

    fn make_token(&mut self, token_type: TokenType, length: usize) -> Result<Token> {
        let token = Token::new(token_type, self.cursor, self.line, self.line_offset, length);
        self.step(length);
        Ok(token)
    }

    fn make_error(&mut self, kind: LexicalErrorKind) -> LexicalError {
        let error = LexicalError::new(kind, self.cursor, self.line, self.line_offset);
        self.step(1);
        error
    }

    fn scan_next(&mut self) -> Result<Token> {
        if self.at_end() {
            return self.make_token(TokenType::EOF, 0);
        }
        let chr = self.source[self.cursor] as char;
        match chr {
            // Ignored characters
            ' ' | '\t' | '\r' => {
                self.step(1);
                // TODO: test for recursion limit exceeded
                self.scan_next()
            }
            '\n' => {
                self.new_line();
                // TODO: test for recursion limit exceeded
                self.scan_next()
            }
            // Single character tokens.
            '(' => self.make_token(TokenType::LeftParen, 1),
            ')' => self.make_token(TokenType::RightParen, 1),
            '{' => self.make_token(TokenType::LeftBrace, 1),
            '}' => self.make_token(TokenType::RightBrace, 1),
            ',' => self.make_token(TokenType::Comma, 1),
            '.' => self.make_token(TokenType::Dot, 1),
            '-' => self.make_token(TokenType::Minus, 1),
            '+' => self.make_token(TokenType::Plus, 1),
            ';' => self.make_token(TokenType::SemiColon, 1),
            '*' => self.make_token(TokenType::Star, 1),
            '?' => self.make_token(TokenType::Qmark, 1),
            ':' => self.make_token(TokenType::Colon, 1),
            '%' => self.make_token(TokenType::Modulo, 1),

            // One or more characters tokens
            '!' => match self.look_ahead() {
                '=' => self.make_token(TokenType::BangEq, 2),
                _ => self.make_token(TokenType::Bang, 1),
            },
            '=' => match self.look_ahead() {
                '=' => self.make_token(TokenType::EqEq, 2),
                _ => self.make_token(TokenType::Equal, 1),
            },
            '>' => match self.look_ahead() {
                '=' => self.make_token(TokenType::GreaterThanEq, 2),
                _ => self.make_token(TokenType::GreaterThan, 1),
            },
            '<' => match self.look_ahead() {
                '=' => self.make_token(TokenType::LessThanEq, 2),
                _ => self.make_token(TokenType::LessThan, 1),
            },
            '/' if self.look_ahead() == '/' => {
                self.scan_comment();
                // TODO: Test for recursion limit exceeded error
                // this is possibily a poor choice
                self.scan_next()
            }
            '/' if self.look_ahead() == '*' => {
                self.scan_block_comment()?;
                // TODO: test for recursion limit exceeded error
                self.scan_next()
            }
            '/' => self.make_token(TokenType::Slash, 1),
            '"' => self.scan_string(),
            '0'..='9' => self.scan_number(),
            '_' | 'a'..='z' | 'A'..='Z' => self.scan_identifier(),

            _ => Err(self.make_error(LexicalErrorKind::UnrecognizedLiterl)),
        }
    }

    fn scan_comment(&mut self) {
        // Maintain a local cursor to index the source code
        // to avoid overhead of repeated calls to self.step(1)
        // and unnecessarily repeatedly switching stack frames.
        // `self.cursor + 2` to skip leading double slash.
        let mut local_cursor = self.cursor + 2;
        // While not at the end of file
        while local_cursor <= self.source_size {
            if self.source[local_cursor] == b'\n' {
                self.step(local_cursor - self.cursor);
                self.new_line();
                break;
            }
            local_cursor += 1;
        }
    }

    fn scan_block_comment(&mut self) -> Result<()> {
        let mut block_count = 1;
        let start_cursor = self.cursor;
        let start_line = self.line;
        let start_line_offset = self.line_offset;
        // Skip initial `/*`
        let mut local_cursor = self.cursor + 2;
        while block_count > 0 && local_cursor < self.source_size - 1 {
            let chr = self.source[local_cursor] as char;
            let next_chr = self.source[local_cursor + 1] as char;
            if chr == '*' && next_chr == '/' {
                block_count -= 1;
                local_cursor += 2;
            } else if chr == '/' && next_chr == '*' {
                block_count += 1;
                local_cursor += 2;
            } else if chr == '\n' {
                // Move the cursor to the end of line
                // and start a new line.
                self.step(local_cursor - self.cursor);
                self.new_line();
                local_cursor += 1;
            } else {
                local_cursor += 1;
            }
        }

        self.step(local_cursor - self.cursor);
        if block_count != 0 {
            Err(LexicalError::new(
                LexicalErrorKind::UnbalancedBlockComment,
                start_cursor,
                start_line,
                start_line_offset,
            ))
        } else {
            Ok(())
        }
    }

    fn scan_string(&mut self) -> Result<Token> {
        let string_start_cursor = self.cursor;
        let string_start_line = self.line;
        let string_start_line_offset = self.line_offset;
        self.step(1);
        loop {
            if self.at_end() {
                break Err(self.make_error(LexicalErrorKind::UnterminatedString));
            }
            let chr = self.source[self.cursor] as char;
            if chr == '"' {
                self.step(1);
                break Ok(Token::new(
                    TokenType::String,
                    string_start_cursor,
                    string_start_line,
                    string_start_line_offset,
                    self.cursor - string_start_cursor, // String length
                ));
            } else {
                // TODO: do something about these repeated calls..
                self.step(1);
            }
        }
    }

    fn scan_number(&mut self) -> Result<Token> {
        let number_start_cursor = self.cursor;
        let number_start_line_offset = self.line_offset;
        let mut dot_consumed = false;
        // Consume the initial digit
        self.step(1);
        while !self.at_end() {
            let chr = self.source[self.cursor] as char;
            match chr {
                '0'..='9' => self.step(1),
                '.' if !dot_consumed => {
                    dot_consumed = true;
                    match self.look_ahead() {
                        '0'..='9' => self.step(2),
                        _ => break,
                    }
                }
                _ => break,
            }
        }

        Ok(Token::new(
            TokenType::Number,
            number_start_cursor,
            self.line,
            number_start_line_offset,
            self.cursor - number_start_cursor,
        ))
    }

    fn scan_identifier(&mut self) -> Result<Token> {
        let mut local_cursor = self.cursor + 1;
        let identifier_start_cursor = self.cursor;
        let identifier_start_line_offset = self.line_offset;
        while !self.at_end() {
            let chr = self.source[local_cursor] as char;
            match chr {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => local_cursor += 1,
                _ => break,
            }
        }
        let lexeme: &[u8] = &self.source[self.cursor..local_cursor];
        let token_type: TokenType = self.get_identifier_type(lexeme)?;
        self.step(local_cursor - self.cursor);
        Ok(Token::new(
            token_type,
            identifier_start_cursor,
            self.line,
            identifier_start_line_offset,
            self.cursor - identifier_start_cursor,
        ))
    }

    fn get_identifier_type(&self, lexeme: &[u8]) -> Result<TokenType> {
        let lexeme: &str = std::str::from_utf8(lexeme).map_err(|_| {
            LexicalError::new(
                LexicalErrorKind::IllegalIdentifer,
                self.cursor,
                self.line,
                self.line_offset,
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
            "class" => TokenType::Class,
            "var" => TokenType::Var,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "fun" => TokenType::Fun,
            "this" => TokenType::This,
            _ => TokenType::Identifier,
        };
        Ok(token_type)
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.closed {
            return None;
        }
        match self.scan_next() {
            Ok(token) => match token.token_type {
                TokenType::EOF => {
                    self.closed = true;
                    Some(Ok(token))
                }
                _ => Some(Ok(token)),
            },
            err => Some(err),
        }
    }
}
