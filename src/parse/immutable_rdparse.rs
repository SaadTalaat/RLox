use super::ast::Expr;
use crate::lex::{Token, TokenType};

pub struct RDParser;

/// Stateless version of the Recursive Descent Parser
impl RDParser {
    fn expect<'a>(token: &'a Token, token_type: TokenType) {
        // TODO: Handle this, don't panic!
        if token_type != token.token_type {
            panic!(
                "Invalid token type: expected {}, found {}",
                token_type, token.token_type
            );
        }
    }

    pub fn parse<'a>(tokens: &'a Vec<Token>) -> Vec<Expr<'a>> {
        let mut cursor = 0;
        let mut expressions = vec![];
        while tokens[cursor].token_type != TokenType::EOF {
            //println!("{}", consumed);
            let (expr, consumed) = Self::expression(&tokens, cursor);
            cursor += consumed;
            expressions.push(expr);
            Self::expect(&tokens[cursor], TokenType::SemiColon);
            cursor += 1;
        }
        expressions
    }

    fn expression<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> (Expr<'a>, usize) {
        Self::equality(&tokens, cursor)
    }

    fn equality<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> (Expr<'a>, usize) {
        let (left, consumed) = Self::comparison(tokens, cursor);
        let cursor = cursor + consumed;
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::EqEq | TokenType::BangEq => {
                let operator = token;
                let (right, consumed2) = Self::equality(&tokens, cursor + 1);
                (
                    Expr::binary(left, &operator, right),
                    consumed + consumed2 + 1,
                )
            }
            _ => (left, consumed),
        }
    }

    fn comparison<'a>(
        tokens: &'a Vec<Token>,
        cursor: usize,
    ) -> (Expr<'a>, usize) {
        let (left, consumed) = Self::term(tokens, cursor);
        let cursor = cursor + consumed;
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::LessThan
            | TokenType::LessThanEq
            | TokenType::GreaterThan
            | TokenType::GreaterThanEq => {
                let operator = token;
                let (right, consumed2) = Self::comparison(&tokens, cursor + 1);
                (
                    Expr::binary(left, &operator, right),
                    consumed + consumed2 + 1,
                )
            }
            _ => (left, consumed),
        }
    }

    fn term<'a>(tokens: &'a Vec<Token>, cursor: usize) -> (Expr<'a>, usize) {
        let (left, consumed) = Self::factor(tokens, cursor);
        let cursor = cursor + consumed;
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::Plus | TokenType::Minus => {
                let operator = token;
                let (right, consumed2) = Self::term(&tokens, cursor + 1);
                (
                    Expr::binary(left, &operator, right),
                    consumed + consumed2 + 1,
                )
            }
            _ => (left, consumed),
        }
    }

    fn factor<'a>(tokens: &'a Vec<Token>, cursor: usize) -> (Expr<'a>, usize) {
        let (left, consumed) = Self::unary(tokens, cursor);
        let cursor = cursor + consumed;
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::Slash | TokenType::Star | TokenType::Modulo => {
                let operator = token;
                let (right, consumed2) = Self::factor(&tokens, cursor + 1);
                (
                    Expr::binary(left, &operator, right),
                    consumed + consumed2 + 1,
                )
            }
            _ => (left, consumed),
        }
    }

    fn unary<'a>(tokens: &'a Vec<Token>, cursor: usize) -> (Expr<'a>, usize) {
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::Bang | TokenType::Minus => {
                let operator = token;
                let (expr, consumed) = Self::unary(tokens, cursor + 1);
                (Expr::unary(&operator, expr), consumed + 1)
            }
            _ => Self::primary(tokens, cursor),
        }
    }

    fn primary<'a>(tokens: &'a Vec<Token>, cursor: usize) -> (Expr<'a>, usize) {
        let token = &tokens[cursor];
        match token.token_type {
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::String => (Expr::literal(&token.value), 1),
            TokenType::LeftParen => {
                let (expr, consumed) = Self::expression(&tokens, cursor + 1);
                Self::expect(
                    &tokens[cursor + consumed + 1],
                    TokenType::RightParen,
                );
                (Expr::grouping(expr), consumed + 2)
            }
            // Handle this, don't panic.
            _ => panic!("Unexpected token: {:?}", token),
        }
    }
}
