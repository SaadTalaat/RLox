use super::ast::Expr;
use super::error::ParseError;
use super::result::Result as ParseResult;
use crate::lex::{Token, TokenType};

pub struct RDParser<'a> {
    current: usize,
    tokens: &'a [Token<'a>],
}

impl<'a> RDParser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self { current: 0, tokens }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        (!self.at_end()) && self.current().token_type == *token_type
    }

    fn current(&self) -> &'a Token<'a> {
        &self.tokens[self.current]
    }
    fn previous(&self) -> &'a Token<'a> {
        &self.tokens[self.current - 1]
    }

    fn at_end(&self) -> bool {
        let token = self.current();
        token.token_type == TokenType::EOF
    }

    fn advance(&mut self) -> &'a Token<'a> {
        self.current = self.current + 1;
        self.previous()
    }

    fn consume(&mut self, token_type: TokenType) -> ParseResult<&Token<'a>> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            // TODO: Actually handle this
            Err(ParseError::new(
                self.current(),
                format!(
                    "Cannot consume token type: {}, found: {}",
                    token_type,
                    self.current().token_type
                ),
            ))
        }
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Expr<'a>>> {
        let mut exprs: Vec<Expr> = Vec::with_capacity(10_000_000);
        while !self.at_end() {
            let expr = self.expression()?;
            exprs.push(expr);
            self.consume(TokenType::SemiColon)?;
        }
        Ok(exprs)
    }

    fn expression(&mut self) -> ParseResult<Expr<'a>> {
        self.equality()
    }

    fn equality(&mut self) -> ParseResult<Expr<'a>> {
        let mut left = self.comparison()?;
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::EqEq | TokenType::BangEq => {
                    let operator = token;
                    self.advance();
                    let right = self.comparison()?;
                    left = Expr::binary(left, &operator, right);
                }
                _ => break Ok(left),
            }
        }
    }

    fn comparison(&mut self) -> ParseResult<Expr<'a>> {
        let mut left = self.term()?;
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::LessThan
                | TokenType::LessThanEq
                | TokenType::GreaterThan
                | TokenType::GreaterThanEq => {
                    let operator = token;
                    self.advance();
                    let right = self.term()?;
                    left = Expr::binary(left, operator, right)
                }
                _ => break Ok(left),
            }
        }
    }

    fn term(&mut self) -> ParseResult<Expr<'a>> {
        let mut left = self.factor()?;
        loop {
            let token = self.current();
            match &token.token_type {
                TokenType::Plus | TokenType::Minus => {
                    let operator = token;
                    self.advance();
                    let right = self.factor()?;
                    left = Expr::binary(left, &operator, right);
                }
                _ => break Ok(left),
            }
        }
    }

    fn factor(&mut self) -> ParseResult<Expr<'a>> {
        let mut left = self.unary()?;
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::Slash | TokenType::Star | TokenType::Modulo => {
                    let operator = token;
                    self.advance();
                    let right = self.unary()?;
                    left = Expr::binary(left, &operator, right);
                }
                _ => break Ok(left),
            }
        }
    }

    fn unary(&mut self) -> ParseResult<Expr<'a>> {
        let token = self.current();
        match token.token_type {
            TokenType::Bang | TokenType::Minus => {
                let operator = token;
                self.advance();
                let expr = self.unary()?;
                Ok(Expr::unary(&operator, expr))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> ParseResult<Expr<'a>> {
        let token = self.current();
        let expr = match token.token_type {
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::String => Ok(Expr::literal(&token.value)),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen)?;
                Ok(Expr::grouping(expr))
            }
            _ => Err(ParseError::new(
                token,
                format!("Unexpected token: {}", token),
            )),
        };
        self.advance();
        expr
    }
}
