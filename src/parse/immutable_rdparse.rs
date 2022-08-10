use super::ast::{Expr, Stmt};
use super::error::ParseError;
use super::Result;
use crate::lex::{Token, TokenType};

pub struct RDParser;
/// Stateless version of the Recursive Descent Parser
impl RDParser {
    fn expect<'a>(token: &'a Token, token_type: TokenType) -> Result<'a, ()> {
        if token_type != token.token_type {
            Err(ParseError::new(
                token,
                format!("expected: {}, found: {}", token_type, token.token_type,),
            ))
        } else {
            Ok(())
        }
    }

    pub fn parse<'a>(tokens: &'a Vec<Token>) -> Result<'a, Vec<Stmt<'a>>> {
        let mut cursor = 0;
        let mut stmts: Vec<Stmt> = Vec::with_capacity(10_000_000);
        while tokens[cursor].token_type != TokenType::EOF {
            let (stmt, consumed) = Self::statement(&tokens, cursor)?;
            cursor += consumed;
            stmts.push(stmt);
        }
        Ok(stmts)
    }

    fn statement<'a>(tokens: &'a Vec<Token>, mut cursor: usize) -> Result<'a, (Stmt<'a>, usize)> {
        let token = &tokens[cursor];
        let start_cursor = cursor;
        match token.token_type {
            TokenType::Print => {
                cursor += 1;
                let (expr, consumed) = Self::expression(tokens, cursor)?;
                cursor += consumed;
                Self::expect(&tokens[cursor], TokenType::SemiColon)?;
                cursor += 1;
                Ok((Stmt::print(expr), cursor - start_cursor))
            }
            _ => {
                let (expr, consumed) = Self::expression(tokens, cursor)?;
                cursor += consumed;
                Self::expect(&tokens[cursor], TokenType::SemiColon)?;
                cursor += 1;
                Ok((Stmt::expr(expr), cursor - start_cursor))
            }
        }
    }

    fn expression<'a>(tokens: &'a Vec<Token>, cursor: usize) -> Result<'a, (Expr<'a>, usize)> {
        Self::ternary(&tokens, cursor)
    }

    fn ternary<'a>(tokens: &'a Vec<Token>, cursor: usize) -> Result<'a, (Expr<'a>, usize)> {
        let (root, consumed) = Self::equality(tokens, cursor)?;
        let start_cursor = cursor;
        let mut cursor = cursor + consumed;
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::Qmark => {
                let (left_operand, consumed_left) = Self::equality(tokens, cursor + 1)?;
                // +1 to account for the Qmark
                cursor += consumed_left + 1;
                Self::expect(&tokens[cursor], TokenType::Colon)?;
                let (right_operand, consumed_right) = Self::ternary(tokens, cursor + 1)?;
                // +1 to account for the Colon
                cursor += consumed_right + 1;
                let expr = Expr::ternary(root, left_operand, right_operand);
                Ok((expr, cursor - start_cursor))
            }
            _ => Ok((root, consumed)),
        }
    }

    fn equality<'a>(tokens: &'a Vec<Token>, cursor: usize) -> Result<'a, (Expr<'a>, usize)> {
        let (mut left, mut consumed_left) = Self::comparison(tokens, cursor)?;
        let mut cursor = cursor + consumed_left;
        loop {
            let token = &tokens[cursor];
            match token.token_type {
                TokenType::EqEq | TokenType::BangEq => {
                    let operator = token;
                    let (right, consumed_right) = Self::equality(&tokens, cursor + 1)?;
                    left = Expr::binary(left, &operator, right);
                    consumed_left += consumed_right + 1;
                    cursor += consumed_right + 1;
                }
                _ => break Ok((left, consumed_left)),
            }
        }
    }

    fn comparison<'a>(tokens: &'a Vec<Token>, cursor: usize) -> Result<'a, (Expr<'a>, usize)> {
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
                    let (right, consumed_right) = Self::term(&tokens, cursor + 1)?;
                    left = Expr::binary(left, &operator, right);
                    consumed_left += consumed_right + 1;
                    cursor += consumed_right + 1;
                }
                _ => break Ok((left, consumed_left)),
            }
        }
    }

    fn term<'a>(tokens: &'a Vec<Token>, cursor: usize) -> Result<'a, (Expr<'a>, usize)> {
        let (mut left, mut consumed_left) = Self::factor(tokens, cursor)?;
        let mut cursor = cursor + consumed_left;
        loop {
            let token = &tokens[cursor];
            match token.token_type {
                TokenType::Plus | TokenType::Minus => {
                    let operator = token;
                    let (right, consumed_right) = Self::factor(&tokens, cursor + 1)?;
                    left = Expr::binary(left, &operator, right);
                    consumed_left += consumed_right + 1;
                    cursor += consumed_right + 1;
                }
                _ => break Ok((left, consumed_left)),
            }
        }
    }

    fn factor<'a>(tokens: &'a Vec<Token>, cursor: usize) -> Result<'a, (Expr<'a>, usize)> {
        let (mut left, mut consumed_left) = Self::unary(tokens, cursor)?;
        let mut cursor = cursor + consumed_left;
        loop {
            let token = &tokens[cursor];
            match token.token_type {
                TokenType::Slash | TokenType::Star | TokenType::Modulo => {
                    let operator = token;
                    let (right, consumed_right) = Self::unary(&tokens, cursor + 1)?;
                    left = Expr::binary(left, &operator, right);
                    consumed_left += consumed_right + 1;
                    cursor += consumed_right + 1;
                }
                _ => break Ok((left, consumed_left)),
            }
        }
    }

    fn unary<'a>(tokens: &'a Vec<Token>, cursor: usize) -> Result<'a, (Expr<'a>, usize)> {
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

    fn primary<'a>(tokens: &'a Vec<Token>, cursor: usize) -> Result<'a, (Expr<'a>, usize)> {
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::String => Ok((Expr::literal(&token.value), 1)),
            TokenType::LeftParen => {
                let (expr, consumed) = Self::expression(&tokens, cursor + 1)?;
                Self::expect(&tokens[cursor + consumed + 1], TokenType::RightParen)?;
                Ok((Expr::grouping(expr), consumed + 2))
            }
            _ => Err(ParseError::new(
                token,
                format!("Unexpected token: {}", token),
            )),
        }
    }
}
