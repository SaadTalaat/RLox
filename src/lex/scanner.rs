use super::error::LexicalError;
use super::result::Result;
use super::token::{Token, TokenType};

pub struct Scanner;

/// Scanner
/// A stateless implementation to avoid maintaining mutable
/// cursors on instaces enforcing a `let mut scanner = ...`
/// declaration.
impl Scanner {
    // Returns a character ahead of the cursor given
    // Example:
    // source = "test"
    // cursor = 0
    // result = 'e'
    fn look_ahead(source: &[u8], cursor: usize) -> char {
        if cursor < (source.len() - 1) {
            source[cursor + 1] as char
        } else {
            '\0'
        }
    }

    fn current(source: &[u8], cursor: usize) -> char {
        if cursor < source.len() {
            source[cursor] as char
        } else {
            '\0'
        }
    }

    // Determines identifier type
    // returns an error in case the identifier contains non UTF-8 characters.
    fn identifier_type(word: &[u8], line: usize, line_offset: usize) -> Result<TokenType> {
        let word: &str = std::str::from_utf8(word).map_err(|_| {
            LexicalError::new(
                line,
                line_offset,
                word[0] as char,
                word,
                "Illegal identifier or keyword",
            )
        })?;
        let token_type = match word {
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
            _ => TokenType::Identifier,
        };
        return Ok(token_type);
    }

    // Scans single line comments
    fn scan_comment(source: &[u8], mut cursor: usize) -> &[u8] {
        // Skips the initial double slash "//"
        let start_cursor = cursor;
        cursor += 2;
        let source_len = source.len();
        let at_end = |idx| idx >= source_len;
        while let _chr = Self::current(source, cursor) {
            if _chr == '\n' || at_end(cursor) {
                break;
            }
            cursor += 1
        }
        &source[start_cursor..cursor]
    }

    // Scans nested block comments
    fn scan_block_comment(
        source: &[u8],
        mut cursor: usize,
        line: usize,
        line_offset: usize,
    ) -> Result<&[u8]> {
        // Skip the initial slash + star "/*"
        let start_cursor = cursor;
        cursor += 2;
        let mut cmt_block_count = 1;
        let source_len = source.len();
        let at_end = |idx| idx >= source_len;
        let look_ahead = |_cur| Self::look_ahead(source, _cur);
        while let _chr = Self::current(source, cursor) {
            if at_end(cursor) {
                return Err(LexicalError::new(
                    line,
                    line_offset,
                    source[0] as char,
                    source,
                    "Unbalanced comment block",
                ));
            } else if _chr == '/' && look_ahead(cursor) == '*' {
                cursor += 2;
                cmt_block_count += 1;
            } else if _chr == '*' && look_ahead(cursor) == '/' {
                cursor += 2;
                cmt_block_count -= 1;
            } else {
                cursor += 1;
            }
            // Do we have a balances comment block?
            if cmt_block_count == 0 {
                break;
            }
        }
        Ok(&source[start_cursor..cursor])
    }

    // Scans string literals
    fn scan_string(
        source: &[u8],
        mut cursor: usize,
        line: usize,
        line_offset: usize,
    ) -> Result<&[u8]> {
        // Account for the initial "
        let start_cursor = cursor;
        cursor += 1;
        let source_len = source.len();
        let at_end = |idx| idx >= source_len;
        while let _chr = Self::current(source, cursor) {
            if at_end(cursor) {
                return Err(LexicalError::new(
                    line,
                    line_offset,
                    source[0] as char,
                    source,
                    "unterminated string literal",
                ));
            } else if _chr == '"' {
                cursor += 1;
                break;
            }
            cursor += 1;
        }
        Ok(&source[start_cursor..cursor])
    }

    // Scans number literals
    fn scan_number(source: &[u8], mut cursor: usize) -> &[u8] {
        // Skip initial character
        let start_cursor = cursor;
        cursor += 1;
        // Can only consume one dot (fractional point).
        let mut dot_consumed = false;
        while let _chr = Self::current(source, cursor) {
            match _chr {
                // consume digits
                '0'..='9' => cursor += 1,
                // consume dots if next char is digit.
                '.' if !dot_consumed => {
                    dot_consumed = true;
                    // Take one more step ahead and see if there's
                    // any digits after the dot
                    match Self::look_ahead(source, cursor) {
                        // consume both the dot and the digit
                        '0'..='9' => cursor += 2,
                        _ => break,
                    }
                }
                _ => break,
            }
        }
        &source[start_cursor..cursor]
    }

    // Scans Identifiers and keywords
    fn scan_identifier(source: &[u8], mut cursor: usize) -> &[u8] {
        let start_cursor = cursor;
        cursor += 1;
        while let _chr = Self::current(source, cursor) {
            match _chr {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => cursor += 1,
                _ => break,
            }
        }
        &source[start_cursor..cursor]
    }

    // Scans the entire source code.
    pub fn scan(source_str: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::with_capacity(10_000_000);
        let lines = source_str.lines().collect::<Vec<&str>>();
        let source = source_str.as_bytes();
        // Token specific cursors
        let mut line = 0;
        let mut line_offset = 0;

        // source code global cursors
        let mut cur = 0;
        let size = source.len();
        let at_end = |idx| idx >= size;
        let look_ahead = |_cur| Self::look_ahead(source, _cur);

        while !at_end(cur) {
            // Assume source code is in ASCII.
            let chr = source[cur] as char;
            let (token_type, lexeme) = match chr {
                // Non significant characters
                ' ' | '\r' | '\t' => (None, &source[cur..cur + 1]),
                '\n' => {
                    line += 1;
                    line_offset = 0;
                    (None, &source[cur..cur + 1])
                }
                // One character literals
                '(' => (Some(TokenType::LeftParen), &source[cur..cur + 1]),
                ')' => (Some(TokenType::RightParen), &source[cur..cur + 1]),
                '{' => (Some(TokenType::LeftBrace), &source[cur..cur + 1]),
                '}' => (Some(TokenType::RightBrace), &source[cur..cur + 1]),
                ',' => (Some(TokenType::Comma), &source[cur..cur + 1]),
                '.' => (Some(TokenType::Dot), &source[cur..cur + 1]),
                '-' => (Some(TokenType::Minus), &source[cur..cur + 1]),
                '+' => (Some(TokenType::Plus), &source[cur..cur + 1]),
                ';' => (Some(TokenType::SemiColon), &source[cur..cur + 1]),
                '*' => (Some(TokenType::Star), &source[cur..cur + 1]),
                '%' => (Some(TokenType::Modulo), &source[cur..cur + 1]),
                '?' => (Some(TokenType::Qmark), &source[cur..cur + 1]),
                ':' => (Some(TokenType::Colon), &source[cur..cur + 1]),

                // Conditional expressions.
                '=' => match look_ahead(cur) {
                    '=' => (Some(TokenType::EqEq), &source[cur..cur + 2]),
                    '>' => (Some(TokenType::EqGreaterThan), &source[cur..cur + 2]),
                    _ => (Some(TokenType::Equal), &source[cur..cur + 1]),
                },
                '!' => match look_ahead(cur) {
                    '=' => (Some(TokenType::BangEq), &source[cur..cur + 2]),
                    _ => (Some(TokenType::Bang), &source[cur..cur + 1]),
                },
                '>' => match look_ahead(cur) {
                    '=' => (Some(TokenType::GreaterThanEq), &source[cur..cur + 2]),
                    _ => (Some(TokenType::GreaterThan), &source[cur..cur + 1]),
                },
                '<' => match look_ahead(cur) {
                    '=' => (Some(TokenType::LessThanEq), &source[cur..cur + 2]),
                    _ => (Some(TokenType::LessThan), &source[cur..cur + 1]),
                },

                // Comment
                '/' if look_ahead(cur) == '/' => {
                    let lexeme = Self::scan_comment(&source, cur);
                    (None, lexeme)
                }
                // Block comments /* .... /* ..... */ ...*/
                '/' if look_ahead(cur) == '*' => {
                    let lexeme = Self::scan_block_comment(&source, cur, line, line_offset)?;
                    (None, lexeme)
                }

                // Slash
                '/' => (Some(TokenType::Slash), &source[cur..cur + 1]),

                // Strings
                '"' => {
                    let lexeme = Self::scan_string(&source, cur, line, line_offset)?;
                    (Some(TokenType::String), lexeme)
                }

                // Numbers
                '0'..='9' => {
                    let lexeme = Self::scan_number(&source, cur);
                    (Some(TokenType::Number), lexeme)
                }
                // Keywords & Identifiers
                '_' | 'a'..='z' | 'A'..='Z' => {
                    let lexeme = Self::scan_identifier(&source, cur);
                    let token_type = Self::identifier_type(lexeme, line, line_offset)?;
                    (Some(token_type), lexeme)
                }
                // Unrecognized literal
                _ => {
                    return Err(LexicalError::new(
                        line,
                        line_offset,
                        chr,
                        source,
                        "Unrecognized literal",
                    ))
                }
            };

            let lexeme_size = lexeme.len();
            if let Some(ttype) = token_type {
                tokens.push(Token::new(
                    ttype,
                    lexeme,
                    source_str,
                    lines[line],
                    line,
                    line_offset,
                    cur,
                ));
            }
            cur += lexeme_size;
            line_offset += lexeme_size;
        }
        tokens.push(Token::new(
            TokenType::EOF,
            b"\0",
            source_str,
            lines[line - 1],
            line,
            lines[line - 1].len() - 1,
            cur + 1,
        ));
        Ok(tokens)
    }
}
