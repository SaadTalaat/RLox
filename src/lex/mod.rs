pub mod error;
pub mod result;
pub mod token;
pub use token::{Token, TokenType};

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

    // Determines identifier type
    // returns an error in case the identifier contains non UTF-8 characters.
    fn identifier_type(
        word: &[u8],
        line: usize,
        line_offset: usize,
    ) -> result::Result<TokenType> {
        let word: &str = std::str::from_utf8(word).map_err(|_| {
            error::Error::new(
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
    fn scan_comment(source: &[u8]) -> &[u8] {
        // Skips the initial double slash "//"
        let mut cmt_size = 2;
        let at_end = |idx| idx >= source.len();
        // Cursor at //
        //            ^ at index 1
        // Note: look_ahead(..., cmt_size - 1) because It's treated
        // as an index not size
        while let _chr = Self::look_ahead(source, cmt_size - 1) {
            if _chr == '\n' || at_end(cmt_size) {
                break;
            }
            cmt_size += 1
        }
        &source[..cmt_size]
    }

    // Scans nested block comments
    fn scan_block_comment(
        source: &[u8],
        line: usize,
        line_offset: usize,
    ) -> result::Result<&[u8]> {
        // Skip the initial slash + star "/*"
        let mut cmt_size = 2;
        let mut cmt_block_count = 1;
        let at_end = |idx| idx >= source.len();
        let look_ahead = |_cur| Self::look_ahead(source, _cur);
        // Note: look_ahead(..., cmt_size - 1) because It's treated
        // as an index not size
        while let _chr = Self::look_ahead(source, cmt_size - 1) {
            if at_end(cmt_size) {
                return Err(error::Error::new(
                    line,
                    line_offset,
                    source[0] as char,
                    source,
                    "Unbalanced comment block",
                ));
            } else if _chr == '/' && look_ahead(cmt_size) == '*' {
                cmt_size += 2;
                cmt_block_count += 1;
            } else if _chr == '*' && look_ahead(cmt_size) == '/' {
                cmt_size += 2;
                cmt_block_count -= 1;
            } else {
                cmt_size += 1;
            }
            // Do we have a balances comment block?
            if cmt_block_count == 0 {
                break;
            }
        }
        Ok(&source[..cmt_size])
    }

    // Scans string literals
    fn scan_string(
        source: &[u8],
        line: usize,
        line_offset: usize,
    ) -> result::Result<&[u8]> {
        // Account for the initial "
        let mut str_size = 1;
        let at_end = |idx| idx >= source.len();
        // Note: look_ahead(..., str_size - 1) because It's treated
        // as an index not size
        while let _chr = Self::look_ahead(source, str_size - 1) {
            if at_end(str_size) {
                return Err(error::Error::new(
                    line,
                    line_offset,
                    source[0] as char,
                    source,
                    "unterminated string literal",
                ));
            } else if _chr == '"' {
                str_size += 1;
                break;
            }
            str_size += 1;
        }
        Ok(&source[..str_size])
    }

    // Scans number literals
    fn scan_number(source: &[u8]) -> &[u8] {
        // Skip initial character
        let mut num_size = 1;
        // Can only consume one dot (fractional point).
        let mut dot_consumed = false;
        // Note: look_ahead(..., num_size - 1) because It's treated
        // as an index not size
        while let _chr = Self::look_ahead(source, num_size - 1) {
            match _chr {
                // consume digits
                '0'..='9' => num_size += 1,
                // consume dots if next char is digit.
                '.' if !dot_consumed => {
                    dot_consumed = true;
                    // Take one more step ahead and see if there's
                    // any digits after the dot
                    match Self::look_ahead(source, num_size) {
                        // consume both the dot and the digit
                        '0'..='9' => num_size += 2,
                        _ => break,
                    }
                }
                _ => break,
            }
        }
        &source[..num_size]
    }

    // Scans Identifiers and keywords
    fn scan_identifier(source: &[u8]) -> &[u8] {
        let mut lexeme_size = 1;
        // Note: look_ahead(..., num_size - 1) because It's treated
        // as an index not size
        while let _chr = Self::look_ahead(source, lexeme_size - 1) {
            match _chr {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => lexeme_size += 1,
                _ => break,
            }
        }
        &source[..lexeme_size]
    }

    // Scans the entire source code.
    pub fn scan(source: &[u8]) -> result::Result<Vec<Token>> {
        let mut tokens = vec![];
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

                // Conditional expressions.
                '=' => match look_ahead(cur) {
                    '=' => (Some(TokenType::EqEq), &source[cur..cur + 2]),
                    '>' => {
                        (Some(TokenType::EqGreaterThan), &source[cur..cur + 2])
                    }
                    _ => (Some(TokenType::Equal), &source[cur..cur + 1]),
                },
                '!' => match look_ahead(cur) {
                    '=' => (Some(TokenType::BangEq), &source[cur..cur + 2]),
                    _ => (Some(TokenType::Bang), &source[cur..cur + 1]),
                },
                '>' => match look_ahead(cur) {
                    '=' => {
                        (Some(TokenType::GreaterThanEq), &source[cur..cur + 2])
                    }
                    _ => (Some(TokenType::GreaterThan), &source[cur..cur + 1]),
                },
                '<' => match look_ahead(cur) {
                    '=' => (Some(TokenType::LessThanEq), &source[cur..cur + 2]),
                    _ => (Some(TokenType::LessThan), &source[cur..cur + 1]),
                },

                // Comment
                '/' if look_ahead(cur) == '/' => {
                    let lexeme = Self::scan_comment(&source[cur..]);
                    (None, lexeme)
                }
                // Block comments /* .... /* ..... */ ...*/
                '/' if look_ahead(cur) == '*' => {
                    let lexeme = Self::scan_block_comment(
                        &source[cur..],
                        line,
                        line_offset,
                    )?;
                    (None, lexeme)
                }

                // Slash
                '/' => (Some(TokenType::Slash), &source[cur..cur + 1]),

                // Strings
                '"' => {
                    let lexeme =
                        Self::scan_string(&source[cur..], line, line_offset)?;
                    (Some(TokenType::String), lexeme)
                }

                // Numbers
                '0'..='9' => {
                    let lexeme = Self::scan_number(&source[cur..]);
                    (Some(TokenType::Number), lexeme)
                }
                // Keywords & Identifiers
                '_' | 'a'..='z' | 'A'..='Z' => {
                    let lexeme = Self::scan_identifier(&source[cur..]);
                    let token_type =
                        Self::identifier_type(lexeme, line, line_offset)?;
                    (Some(token_type), lexeme)
                }
                // Unrecognized literal
                _ => {
                    return Err(error::Error::new(
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
                tokens.push(Token::new(ttype, lexeme, line, line_offset, cur));
            }
            cur += lexeme_size;
            line_offset += lexeme_size;
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests;
