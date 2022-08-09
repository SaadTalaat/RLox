use super::ast::Expr;
use super::error::ParseError;
use super::result::Result as ParseResult;
use crate::lex::{Token, TokenType};

pub struct RDParser;
/// Stateless version of the Recursive Descent Parser
impl RDParser {
    fn expect<'a>(token: &'a Token, token_type: TokenType) -> ParseResult<()> {
        // TODO: Handle this, don't panic!
        if token_type != token.token_type {
            Err(ParseError::new(
                token,
                format!(
                    "Invalid token type: expected {}, found {}",
                    token_type, token.token_type,
                ),
            ))
        } else {
            Ok(())
        }
    }

    pub fn parse<'a>(tokens: &'a Vec<Token>) -> ParseResult<Vec<Expr<'a>>> {
        let mut cursor = 0;
        let mut exprs: Vec<Expr> = Vec::with_capacity(10_000_000);
        while tokens[cursor].token_type != TokenType::EOF {
            let (expr, consumed) = Self::expression(&tokens, cursor)?;
            cursor += consumed;
            exprs.push(expr);
            Self::expect(&tokens[cursor], TokenType::SemiColon)?;
            cursor += 1;
        }
        Ok(exprs)
    }

    fn expression<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> ParseResult<(Expr<'a>, usize)> {
        Self::equality(&tokens, cursor)
    }

    fn equality<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> ParseResult<(Expr<'a>, usize)> {
        let (mut left, mut consumed_left) = Self::comparison(tokens, cursor)?;
        let mut cursor = cursor + consumed_left;
        loop {
            let token = &tokens[cursor];
            match token.token_type {
                TokenType::EqEq | TokenType::BangEq => {
                    let operator = token;
                    let (right, consumed_right) =
                        Self::equality(&tokens, cursor + 1)?;
                    left = Expr::binary(left, &operator, right);
                    consumed_left += consumed_right + 1;
                    cursor += consumed_right + 1;
                }
                _ => break Ok((left, consumed_left)),
            }
        }
    }

    fn comparison<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> ParseResult<(Expr<'a>, usize)> {
        let (mut left, mut consumed_left) = Self::term(tokens, cursor)?;
        let mut cursor = cursor + consumed_left;
        loop {
            let token = &tokens[cursor];
            match token.token_type {
                TokenType::LessThan
                | TokenType::LessThanEq
                | TokenType::GreaterThan
                | TokenType::GreaterThanEq => {
                    let operator = token;
                    let (right, consumed_right) =
                        Self::term(&tokens, cursor + 1)?;
                    left = Expr::binary(left, &operator, right);
                    consumed_left += consumed_right + 1;
                    cursor += consumed_right + 1;
                }
                _ => break Ok((left, consumed_left)),
            }
        }
    }

    fn term<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> ParseResult<(Expr<'a>, usize)> {
        let (mut left, mut consumed_left) = Self::factor(tokens, cursor)?;
        let mut cursor = cursor + consumed_left;
        loop {
            let token = &tokens[cursor];
            match token.token_type {
                TokenType::Plus | TokenType::Minus => {
                    let operator = token;
                    let (right, consumed_right) =
                        Self::factor(&tokens, cursor + 1)?;
                    left = Expr::binary(left, &operator, right);
                    consumed_left += consumed_right + 1;
                    cursor += consumed_right + 1;
                }
                _ => break Ok((left, consumed_left)),
            }
        }
    }

    fn factor<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> ParseResult<(Expr<'a>, usize)> {
        let (mut left, mut consumed_left) = Self::unary(tokens, cursor)?;
        let mut cursor = cursor + consumed_left;
        loop {
            let token = &tokens[cursor];
            match token.token_type {
                TokenType::Slash | TokenType::Star | TokenType::Modulo => {
                    let operator = token;
                    let (right, consumed_right) =
                        Self::unary(&tokens, cursor + 1)?;
                    left = Expr::binary(left, &operator, right);
                    consumed_left += consumed_right + 1;
                    cursor += consumed_right + 1;
                }
                _ => break Ok((left, consumed_left)),
            }
        }
    }

    fn unary<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> ParseResult<(Expr<'a>, usize)> {
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::Bang | TokenType::Minus => {
                let operator = token;
                let (expr, consumed) = Self::unary(tokens, cursor + 1)?;
                Ok((Expr::unary(&operator, expr), consumed + 1))
            }
            _ => Self::primary(tokens, cursor),
        }
    }

    fn primary<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> ParseResult<(Expr<'a>, usize)> {
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::String => Ok((Expr::literal(&token.value), 1)),
            TokenType::LeftParen => {
                let (expr, consumed) = Self::expression(&tokens, cursor + 1)?;
                Self::expect(
                    &tokens[cursor + consumed + 1],
                    TokenType::RightParen,
                )?;
                Ok((Expr::grouping(expr), consumed + 2))
            }
            // Handle this, don't panic.
            _ => Err(ParseError::new(
                token,
                format!("Unexpected token: {}", token),
            )),
        }
    }
}
