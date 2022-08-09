use super::ast::Expr;
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

    fn consume(&mut self, token_type: TokenType) -> &'a Token<'a> {
        if self.check(&token_type) {
            self.advance()
        } else {
            // TODO: Actually handle this
            panic!(
                "Cannot consume token type: {}, found: {}",
                token_type,
                self.current().token_type
            );
        }
    }

    pub fn parse(&mut self) -> Vec<Expr<'a>> {
        let mut exprs = vec![];
        while !self.at_end() {
            let expr = self.expression();
            exprs.push(expr);
            self.consume(TokenType::SemiColon);
        }
        exprs
    }

    fn expression(&mut self) -> Expr<'a> {
        self.equality()
    }

    fn equality(&mut self) -> Expr<'a> {
        let left = self.comparison();
        let token = self.current();
        match token.token_type {
            TokenType::EqEq | TokenType::BangEq => {
                let operator = token;
                self.advance();
                let right = self.equality();
                Expr::binary(left, &operator, right)
            }
            _ => left,
        }
    }

    fn comparison(&mut self) -> Expr<'a> {
        let left = self.term();
        let token = self.current();
        match token.token_type {
            TokenType::LessThan
            | TokenType::LessThanEq
            | TokenType::GreaterThan
            | TokenType::GreaterThanEq => {
                let operator = token;
                self.advance();
                let right = self.comparison();
                Expr::binary(left, operator, right)
            }
            _ => left,
        }
    }

    fn term(&mut self) -> Expr<'a> {
        let left = self.factor();
        let token = self.current();
        match &token.token_type {
            TokenType::Plus | TokenType::Minus => {
                let operator = token;
                self.advance();
                let right = self.term();
                Expr::binary(left, &operator, right)
            }
            _ => left,
        }
    }

    fn factor(&mut self) -> Expr<'a> {
        let left = self.unary();
        let token = self.current();
        match token.token_type {
            TokenType::Slash | TokenType::Star | TokenType::Modulo => {
                let operator = token;
                self.advance();
                let right = self.factor();
                Expr::binary(left, &operator, right)
            }
            _ => left,
        }
    }

    fn unary(&mut self) -> Expr<'a> {
        let token = self.current();
        match token.token_type {
            TokenType::Bang | TokenType::Minus => {
                let operator = token;
                self.advance();
                let expr = self.unary();
                Expr::unary(&operator, expr)
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Expr<'a> {
        let token = self.current();
        let expr = match token.token_type {
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::String => Expr::literal(&token.value),
            TokenType::LeftParen => {
                let expr = self.expression();
                self.consume(TokenType::RightParen);
                Expr::grouping(expr)
            }
            _ => panic!("Unexpected token: {:?}", token),
        };
        self.advance();
        expr
    }
}
