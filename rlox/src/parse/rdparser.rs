use super::ast::{Expr, Operator as ExprOperator, Stmt};
use super::error::{ParseError, ParseErrorKind};
use super::Result;
use crate::code::Code;
use crate::lex::{Token, TokenType};
use crate::LoxValue;

pub struct RDParser<'a> {
    tokens: Vec<Token>,
    code: Code<'a>,
    current: usize,
    loop_depth: usize,
}

impl<'a> RDParser<'a> {
    pub fn new(tokens: Vec<Token>, code: Code<'a>) -> Self {
        Self {
            tokens,
            code,
            current: 0,
            loop_depth: 0,
        }
    }

    fn get_operator(&self, token: &Token) -> Result<ExprOperator> {
        let lexeme = self.code.lexeme(token.location);
        ExprOperator::from_token(token, lexeme)
    }

    fn step(&mut self) {
        self.current += 1;
    }

    fn current(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn at_end(&self) -> bool {
        let token = &self.tokens[self.current];
        token.token_type == TokenType::EOF
    }

    fn synchronize(&mut self) {
        while !self.at_end() {
            let token = self.current();
            match token.token_type {
                TokenType::SemiColon => {
                    self.step();
                    break;
                }
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::If
                | TokenType::For
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => break,
                _ => self.step(),
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, error_kind: ParseErrorKind) -> Result<&Token> {
        if self.current().token_type == token_type {
            self.step();
            Ok(self.previous())
        } else {
            Err(ParseError::new(error_kind, self.current().location))
        }
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let token = self.current();
        match token.token_type {
            TokenType::Var => self.var_declaration(),
            TokenType::Fun => self.func_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        // Consume initial var token.
        self.step();
        self.consume(TokenType::Identifier, ParseErrorKind::MissingVariableName)?;
        let name = self.code.get_identifier(self.previous());
        let init: Option<Expr> = match self.current().token_type {
            TokenType::Equal => {
                self.step();
                Some(self.expression()?)
            }
            _ => None,
        };
        self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
        Ok(Stmt::Var { name, init })
    }

    fn func_declaration(&mut self) -> Result<Stmt> {
        //Skip the fun keyword
        self.step();
        self.consume(TokenType::Identifier, ParseErrorKind::MissingFunctionName)?;
        let name = self.code.get_identifier(self.previous());
        self.consume(TokenType::LeftParen, ParseErrorKind::IllegalFunctionDecl)?;
        let mut params: Vec<String> = vec![];
        let token = self.current();
        if TokenType::RightParen != token.token_type {
            loop {
                self.consume(TokenType::Identifier, ParseErrorKind::ParamExpected)?;
                let param = self.code.get_identifier(self.previous());
                let token = self.current();
                params.push(param);
                if TokenType::Comma == token.token_type {
                    self.step();
                } else {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, ParseErrorKind::UnbalancedParentheses);
        // Parse definition block
        let body = self.block()?;
        Ok(Stmt::function(name, params, body))
    }

    fn statement(&mut self) -> Result<Stmt> {
        let token = self.current();
        match token.token_type {
            TokenType::If => self.if_stmt(),
            TokenType::While => self.while_stmt(),
            TokenType::Break => self.break_stmt(),
            TokenType::Continue => self.continue_stmt(),
            TokenType::For => self.for_stmt(),
            TokenType::Print => self.print_stmt(),
            TokenType::Return => self.return_stmt(),
            TokenType::LeftBrace => self.block(),
            _ => self.expression_stmt(),
        }
    }

    fn block(&mut self) -> Result<Stmt> {
        let mut stmts = vec![];
        self.consume(TokenType::LeftBrace, ParseErrorKind::FatalError)?;
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::RightBrace => break,
                TokenType::EOF => break,
                _ => stmts.push(self.declaration()?),
            }
        }
        self.consume(TokenType::RightBrace, ParseErrorKind::UnterminatedBlock)?;
        Ok(Stmt::Block(stmts))
    }

    fn if_stmt(&mut self) -> Result<Stmt> {
        // skip the leading if token.
        self.step();
        self.consume(TokenType::LeftParen, ParseErrorKind::IllegalIfStatement)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, ParseErrorKind::IllegalIfStatement)?;
        let then = self.statement()?;
        let maybe_else = self.current();
        match maybe_else.token_type {
            TokenType::Else => {
                self.step();
                let otherwise = Some(self.statement()?);
                Ok(Stmt::if_stmt(condition, then, otherwise))
            }
            _ => Ok(Stmt::if_stmt(condition, then, None)),
        }
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        // Skip "while" token.
        self.step();
        self.consume(TokenType::LeftParen, ParseErrorKind::IllegalWhile)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, ParseErrorKind::IllegalWhile)?;
        self.loop_depth += 1;
        let body = self.statement()?;
        self.loop_depth -= 1;
        Ok(Stmt::while_stmt(condition, body))
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        // Skip "for" token.
        self.step();
        self.consume(TokenType::LeftParen, ParseErrorKind::IllegalFor)?;
        // Initializer?
        let token = self.current();
        let initializer: Option<Stmt> = match token.token_type {
            TokenType::Var => Some(self.var_declaration()?),
            TokenType::SemiColon => {
                self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
                None
            }
            _ => Some(self.expression_stmt()?),
        };
        // condition?
        let token = self.current();
        let condition: Option<Expr> = match token.token_type {
            TokenType::SemiColon => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
        // increment?
        let token = self.current();
        let increment: Option<Expr> = match token.token_type {
            TokenType::RightParen => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::RightParen, ParseErrorKind::UnbalancedParentheses)?;
        self.loop_depth += 1;
        let mut body: Stmt = self.statement()?;
        // Desugared
        // {
        //  <init>
        //  while <condition> {
        //   {
        //     <body>
        //   }
        //   <increment>
        //  }
        // }
        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expr(inc)]);
        }
        let cond: Expr = condition.unwrap_or(Expr::literal(LoxValue::Boolean(true)));
        body = Stmt::while_stmt(cond, body);
        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }
        self.loop_depth -= 1;
        Ok(body)
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        self.step();
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
        Ok(Stmt::Print(expr))
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::Return, ParseErrorKind::FatalError)?;
        let token = self.current();
        if TokenType::SemiColon != token.token_type {
            let value = self.expression()?;
            self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
            Ok(Stmt::Return(Some(value)))
        } else {
            self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
            Ok(Stmt::Return(None))
        }
    }

    fn break_stmt(&mut self) -> Result<Stmt> {
        if self.loop_depth == 0 {
            Err(ParseError::new(
                ParseErrorKind::MissingSemiColon,
                self.current().location,
            ))
        } else {
            // Skip the break keyword.
            self.step();
            self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon);
            Ok(Stmt::Break)
        }
    }

    fn continue_stmt(&mut self) -> Result<Stmt> {
        if self.loop_depth == 0 {
            Err(ParseError::new(
                ParseErrorKind::MissingSemiColon,
                self.current().location,
            ))
        } else {
            // Skip the continue keyword.
            self.step();
            self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon);
            Ok(Stmt::Continue)
        }
    }

    fn expression_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let identifier = self.ternary()?;
        let token = self.current();
        match token.token_type {
            TokenType::Equal => match identifier {
                Expr::Var { name } => {
                    self.step();
                    let r_value = self.assignment()?;
                    Ok(Expr::assign(name, r_value))
                }
                _ => Err(ParseError::new(
                    ParseErrorKind::RvToRvAssignment,
                    token.location,
                )),
            },
            _ => Ok(identifier),
        }
    }

    fn ternary(&mut self) -> Result<Expr> {
        let root = self.logical_or()?;
        let token = self.current();
        match token.token_type {
            TokenType::Qmark => {
                self.step();
                let left = self.logical_or()?;
                self.consume(TokenType::Colon, ParseErrorKind::MissingTernaryColon)?;
                let right = self.ternary()?;
                Ok(Expr::ternary(root, left, right))
            }
            _ => Ok(root),
        }
    }

    fn logical_or(&mut self) -> Result<Expr> {
        let left = self.logical_and()?;
        let token = self.current();
        match token.token_type {
            TokenType::Or => {
                let operator: ExprOperator = self.get_operator(token)?;
                self.step();
                let right = self.logical_and()?;
                Ok(Expr::logical(left, operator, right))
            }
            _ => Ok(left),
        }
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let left = self.equality()?;
        let token = self.current();
        match token.token_type {
            TokenType::And => {
                let operator: ExprOperator = self.get_operator(token)?;
                self.step();
                let right = self.equality()?;
                Ok(Expr::logical(left, operator, right))
            }
            _ => Ok(left),
        }
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut left = self.comparison()?;

        // Loop for left-associativity,
        // a == (b == (c != d)) ..etc
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::EqEq | TokenType::BangEq => {
                    let operator: ExprOperator = self.get_operator(token)?;
                    self.step();
                    let right = self.comparison()?;
                    left = Expr::binary(left, operator, right);
                }
                _ => break Ok(left),
            }
        }
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut left = self.term()?;

        // Loop for left-associativity
        // a > (b <= c) ..etc
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::GreaterThan
                | TokenType::GreaterThanEq
                | TokenType::LessThan
                | TokenType::LessThanEq => {
                    let operator: ExprOperator = self.get_operator(token)?;
                    self.step();
                    let right = self.term()?;
                    left = Expr::binary(left, operator, right);
                }
                _ => break Ok(left),
            }
        }
    }

    fn term(&mut self) -> Result<Expr> {
        let mut left = self.factor()?;

        // Loop for left-associativity
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::Plus | TokenType::Minus => {
                    let operator: ExprOperator = self.get_operator(token)?;
                    self.step();
                    let right = self.factor()?;
                    left = Expr::binary(left, operator, right);
                }
                _ => break Ok(left),
            }
        }
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;
        // Loop for left associativity
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::Slash | TokenType::Star | TokenType::Modulo => {
                    let operator: ExprOperator = self.get_operator(token)?;
                    self.step();
                    let right = self.unary()?;
                    left = Expr::binary(left, operator, right);
                }
                _ => break Ok(left),
            }
        }
    }

    fn unary(&mut self) -> Result<Expr> {
        let token = self.current();
        match token.token_type {
            TokenType::Minus | TokenType::Bang => {
                let operator: ExprOperator = self.get_operator(token)?;
                self.step();
                let expr = self.unary()?;
                Ok(Expr::unary(operator, expr))
            }
            _ => self.call(),
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let callee = self.primary()?;
        let token = self.current();
        match token.token_type {
            TokenType::LeftParen => self.finish_call(callee),
            _ => Ok(callee),
        }
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        // Skip the left parentheses
        self.step();
        let mut args: Vec<Expr> = vec![];
        let token = self.current();
        if TokenType::RightParen != token.token_type {
            loop {
                if args.len() >= 255 {
                    return Err(ParseError::new(
                        ParseErrorKind::TooManyArgs,
                        self.current().location,
                    ));
                }
                args.push(self.expression()?);
                let token = self.current();
                if TokenType::Comma == token.token_type {
                    self.step();
                } else {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, ParseErrorKind::UnbalancedParentheses)?;
        Ok(Expr::call_expr(callee, args))
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.current();
        match token.token_type {
            TokenType::False
            | TokenType::True
            | TokenType::Number
            | TokenType::Nil
            | TokenType::String => {
                let literal = Expr::literal(self.code.get_value(token));
                self.step();
                Ok(literal)
            }
            TokenType::LeftParen => {
                self.step();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, ParseErrorKind::UnbalancedParentheses)?;
                Ok(Expr::grouping(expr))
            }
            TokenType::Identifier => {
                let expr = Expr::variable(self.code.get_identifier(token));
                self.step();
                Ok(expr)
            }
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                token.location,
            )),
        }
    }
}

impl Iterator for RDParser<'_> {
    type Item = Result<Stmt>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.at_end() {
            match self.declaration() {
                Ok(stmt) => Some(Ok(stmt)),
                error => {
                    self.synchronize();
                    Some(error)
                }
            }
        } else {
            None
        }
    }
}
