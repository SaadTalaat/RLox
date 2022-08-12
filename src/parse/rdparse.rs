use super::ast::{Expr, Stmt};
use super::error::ParseError;
use super::Result;
use crate::lex::{Token, TokenType};
use crate::LiteralValue;

pub struct RDParser<'a> {
    current: usize,
    loop_depth: usize,
    tokens: &'a [Token<'a>],
}

impl<'a> RDParser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            loop_depth: 0,
            current: 0,
            tokens,
        }
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
            TokenType::Var => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<'a, Stmt<'a>> {
        // Skip the leading `var` token.
        self.advance();
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
            TokenType::Print => self.print_statement(),
            TokenType::LeftBrace => self.block(),
            TokenType::If => self.if_statement(),
            TokenType::While => self.while_statement(),
            TokenType::For => self.for_statement(),
            TokenType::Break => self.break_statement(),
            TokenType::Continue => self.continue_statement(),
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> Result<'a, Stmt<'a>> {
        // Skip the leading `print` token
        self.advance();
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "expected ';' after print statement")?;
        Ok(Stmt::print(expr))
    }

    fn block(&mut self) -> Result<'a, Stmt<'a>> {
        // Skip the leading left brace `{`
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

    fn if_statement(&mut self) -> Result<'a, Stmt<'a>> {
        // Skip the leading `if` token
        self.advance();
        self.consume(TokenType::LeftParen, "expected '(' after an if statement")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "expected ')' after if condition")?;
        let if_body = self.statement()?;
        let maybe_else = self.current();
        match maybe_else.token_type {
            TokenType::Else => {
                self.advance();
                let else_body = self.statement()?;
                Ok(Stmt::if_(condition, if_body, Some(else_body)))
            }
            _ => Ok(Stmt::if_(condition, if_body, None)),
        }
    }

    fn while_statement(&mut self) -> Result<'a, Stmt<'a>> {
        self.loop_depth += 1;
        // Skip the leading `while` token.
        self.advance();
        self.consume(TokenType::LeftParen, "expected '(' after an if statement")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "expected ')' after if condition")?;
        let body = self.statement()?;
        self.loop_depth -= 1;
        Ok(Stmt::while_(condition, body))
    }

    fn for_statement(&mut self) -> Result<'a, Stmt<'a>> {
        // Skip the leading 'for' token
        self.advance();
        self.consume(TokenType::LeftParen, "expected '(' after a for statement")?;
        // Extract the initializer
        let token = self.current();
        let initializer = match token.token_type {
            TokenType::Var => Some(self.var_declaration()?),
            TokenType::SemiColon => None,
            _ => Some(self.expression_statement()?),
        };
        // Extract the condition
        let token = self.current();
        let condition = match token.token_type {
            TokenType::SemiColon => None,
            _ => {
                let condition = self.expression()?;
                self.consume(
                    TokenType::SemiColon,
                    "expected ';' after for-loop condition",
                )?;
                Some(condition)
            }
        };
        // Extract the increment
        let token = self.current();
        let increment = match token.token_type {
            TokenType::RightParen => None,
            _ => {
                let increment = self.expression()?;
                self.consume(
                    TokenType::RightParen,
                    "expected ')' after for-loop incremenet",
                )?;
                Some(Stmt::expr(increment))
            }
        };
        // We're inside the loop now.
        self.loop_depth += 1;
        // Extract the loop body.
        let mut body = self.statement()?;
        // De-sugar the loop
        if let Some(inc) = increment {
            body = Stmt::block(vec![body, inc]);
        }

        if let Some(cond) = condition {
            body = Stmt::while_(cond, body);
        }

        if let Some(init) = initializer {
            body = Stmt::block(vec![init, body]);
        }
        self.loop_depth -= 1;
        Ok(body)
    }

    fn break_statement(&mut self) -> Result<'a, Stmt<'a>> {
        // Skip the `break` token.
        self.advance();
        self.consume(TokenType::SemiColon, "expected ';' after break")?;
        if self.loop_depth == 0 {
            Err(ParseError::new(
                self.current(),
                "break outside of a loop".to_owned(),
            ))
        } else {
            Ok(Stmt::break_())
        }
    }

    fn continue_statement(&mut self) -> Result<'a, Stmt<'a>> {
        // Skip the `continue` token.
        self.advance();
        self.consume(TokenType::SemiColon, "expected ';' after continue")?;
        if self.loop_depth == 0 {
            Err(ParseError::new(
                self.current(),
                "continue outside of a loop".to_owned(),
            ))
        } else {
            Ok(Stmt::continue_())
        }
    }

    fn expression_statement(&mut self) -> Result<'a, Stmt<'a>> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "expected ';' after expression")?;
        Ok(Stmt::expr(expr))
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
        let root = self.logic_or()?;
        let token = self.current();
        match token.token_type {
            TokenType::Qmark => {
                self.advance();
                let left_operand = self.logic_or()?;
                self.consume(TokenType::Colon, "Expected ':' after ternary operator '?'")?;
                let right_operand = self.ternary()?;
                let expr = Expr::ternary(root, left_operand, right_operand);
                Ok(expr)
            }
            _ => Ok(root),
        }
    }

    fn logic_or(&mut self) -> Result<'a, Expr<'a>> {
        let mut left = self.logic_and()?;

        loop {
            let token = self.current();
            match token.token_type {
                TokenType::Or => {
                    let operator = token;
                    self.advance();
                    let right = self.logic_and()?;
                    left = Expr::logical(left, &operator, right);
                }

                _ => break Ok(left),
            }
        }
    }

    fn logic_and(&mut self) -> Result<'a, Expr<'a>> {
        let mut left = self.equality()?;

        loop {
            let token = self.current();
            match token.token_type {
                TokenType::And => {
                    let operator = token;
                    self.advance();
                    let right = self.equality()?;
                    left = Expr::logical(left, &operator, right);
                }

                _ => break Ok(left),
            }
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
            // Ensure followed by a variable expression
            TokenType::MinusMinus | TokenType::PlusPlus => {
                let operator = token;
                self.advance();
                let expr = self.primary()?;
                match expr {
                    Expr::Variable { name } => {
                        // Used to increment and decrement variables
                        let literal_1 = Expr::literal(LiteralValue::Number(1.0));
                        let increment = Expr::binary(expr, operator, literal_1);
                        Ok(Expr::assignment(name, increment))
                    }
                    _ => Err(ParseError::new(
                        operator,
                        "Illegal right operand of prefix expression".to_owned(),
                    )),
                }
            }
            _ => self.postfix_unary(),
        }
    }

    fn postfix_unary(&mut self) -> Result<'a, Expr<'a>> {
        let expr = self.primary()?;
        let token = self.current();
        match token.token_type {
            TokenType::MinusMinus | TokenType::PlusPlus => {
                let operator = token;
                self.advance();
                match expr {
                    Expr::Variable { name } => {
                        let literal_1 = Expr::literal(LiteralValue::Number(1.0));
                        let increment = Expr::binary(expr, operator, literal_1);
                        Ok(Expr::assignment(name, increment))
                    }
                    _ => Err(ParseError::new(
                        token,
                        "Illegal left operand of postfix expression".to_owned(),
                    )),
                }
            }
            _ => Ok(expr),
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
                Ok(Expr::literal(token.value.clone()))
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
