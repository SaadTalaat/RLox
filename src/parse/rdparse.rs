use super::ast::{Expr, Stmt};
use super::error::ParseError;
use super::Result;
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
        self.current += 1;
        self.previous()
    }

    fn consume(&mut self, token_type: TokenType, error_msg: &str) -> Result<'a, &'a Token<'a>> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            // TODO: Actually handle this
            Err(ParseError::new(self.previous(), error_msg.to_owned()))
        }
    }

    pub fn parse(&mut self) -> Result<'a, Vec<Stmt<'a>>> {
        let mut stmts: Vec<Stmt> = Vec::with_capacity(10_000_000);
        while !self.at_end() {
            let stmt = self.declaration()?;
            stmts.push(stmt);
        }
        Ok(stmts)
    }

    fn declaration(&mut self) -> Result<'a, Stmt<'a>> {
        let token = self.current();
        match token.token_type {
            TokenType::Var => {
                self.advance();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<'a, Stmt<'a>> {
        let identifier = self.consume(
            TokenType::Identifier,
            "Expected an identifier after 'var' keyword",
        )?;
        let stmt = match self.current().token_type {
            TokenType::Equal => {
                self.advance();
                let expr = self.expression()?;
                Stmt::variable(identifier, Some(expr))
            }
            _ => Stmt::variable(identifier, None),
        };

        self.consume(
            TokenType::SemiColon,
            "expected ';' after variable declaration",
        )?;

        Ok(stmt)
    }

    fn statement(&mut self) -> Result<'a, Stmt<'a>> {
        let token = self.current();
        match token.token_type {
            TokenType::Print => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::SemiColon, "expected ';' after print statement")?;
                Ok(Stmt::print(expr))
            }
            TokenType::LeftBrace => {
                self.advance();
                let mut stmts = vec![];
                loop {
                    // Look for the closing brace,
                    // if not, read a declaration.
                    let token = self.current();
                    match token.token_type {
                        TokenType::RightBrace => break,
                        TokenType::EOF => break,
                        _ => {
                            let stmt = self.declaration()?;
                            stmts.push(stmt);
                        }
                    }
                }
                self.consume(TokenType::RightBrace, "Expected closing '}'")?;
                Ok(Stmt::block(stmts))
            }
            _ => {
                let expr = self.expression()?;
                self.consume(TokenType::SemiColon, "expected ';' after a statement 2")?;
                Ok(Stmt::expr(expr))
            }
        }
    }

    fn expression(&mut self) -> Result<'a, Expr<'a>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<'a, Expr<'a>> {
        let identifier = self.ternary()?;
        let token = self.current();
        match token.token_type {
            TokenType::Equal => match identifier {
                Expr::Variable { name } => {
                    self.advance();
                    let r_expr = self.assignment()?;
                    let expr = Expr::assignment(name, r_expr);
                    Ok(expr)
                }
                _ => Err(ParseError::new(
                    token,
                    "Invalid assignment target".to_owned(),
                )),
            },
            _ => Ok(identifier),
        }
    }

    fn ternary(&mut self) -> Result<'a, Expr<'a>> {
        let root = self.equality()?;
        let token = self.current();
        match token.token_type {
            TokenType::Qmark => {
                self.advance();
                let left_operand = self.equality()?;
                self.consume(TokenType::Colon, "Expected ':' after ternary operator '?'")?;
                let right_operand = self.ternary()?;
                let expr = Expr::ternary(root, left_operand, right_operand);
                Ok(expr)
            }
            _ => Ok(root),
        }
    }

    fn equality(&mut self) -> Result<'a, Expr<'a>> {
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

    fn comparison(&mut self) -> Result<'a, Expr<'a>> {
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

    fn term(&mut self) -> Result<'a, Expr<'a>> {
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

    fn factor(&mut self) -> Result<'a, Expr<'a>> {
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

    fn unary(&mut self) -> Result<'a, Expr<'a>> {
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

    fn primary(&mut self) -> Result<'a, Expr<'a>> {
        let token = self.current();
        let expr = match token.token_type {
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::String => {
                self.advance();
                Ok(Expr::literal(&token.value))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Mismatched parentheses")?;
                Ok(Expr::grouping(expr))
            }
            TokenType::Identifier => {
                self.advance();
                Ok(Expr::variable(token))
            }
            _ => Err(ParseError::new(
                token,
                format!("Unexpected token: {}", token),
            )),
        };
        expr
    }
}

impl<'a> Iterator for RDParser<'a> {
    type Item = Stmt<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current().token_type != TokenType::EOF {
            match self.statement() {
                Ok(x) => Some(x),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}
