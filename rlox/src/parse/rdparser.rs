use super::ast::{Expr, ExprKind, Operator as ExprOperator, Stmt};
use super::error::{ParseError, ParseErrorKind};
use super::Result;
use crate::code::Code;
use crate::lex::{Token, TokenType};
use crate::LoxValue;

enum FunctionType {
    Function,
    Method,
}

pub struct RDParser<'a> {
    tokens: Vec<Token>,
    code: &'a Code<'a>,
    current: usize,
}

impl<'a> RDParser<'a> {
    pub fn new(tokens: Vec<Token>, code: &'a Code<'a>) -> Self {
        Self {
            tokens,
            code,
            current: 0,
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
            Err(ParseError::new(error_kind, &self.current().location))
        }
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let token = self.current();
        match token.token_type {
            TokenType::Var => self.var_declaration(),
            TokenType::Fun => self.func_declaration(FunctionType::Function),
            TokenType::Class => self.class_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        // Consume initial var token.
        self.consume(TokenType::Var, ParseErrorKind::IllegalVarDeclaration)?;
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
        Ok(Stmt::variable(name, init, self.previous().location))
    }

    fn func_declaration(&mut self, fn_type: FunctionType) -> Result<Stmt> {
        if let FunctionType::Function = fn_type {
            self.consume(TokenType::Fun, ParseErrorKind::FatalError)?;
        }
        let location = self.current().location;
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
        self.consume(TokenType::RightParen, ParseErrorKind::UnbalancedParentheses)?;
        // Parse definition block
        let body = self.block()?;
        Ok(Stmt::function(name, params, body, location))
    }

    fn class_declaration(&mut self) -> Result<Stmt> {
        let location = self.current().location;
        self.consume(TokenType::Class, ParseErrorKind::FatalError)?;
        self.consume(TokenType::Identifier, ParseErrorKind::MissingClassName)?;
        let name = self.code.get_identifier(self.previous());
        // Inherits from another class?
        let base = match self.current().token_type {
            TokenType::LessThan => {
                // Skip the less-than operater.
                self.step();
                self.consume(TokenType::Identifier, ParseErrorKind::IllegalClassDecl)?;
                let super_cls = self.code.get_identifier(self.previous());
                Some(Expr::variable(super_cls, self.previous().location))
            }
            _ => None,
        };

        self.consume(TokenType::LeftBrace, ParseErrorKind::IllegalClassDecl)?;
        let mut methods: Vec<Stmt> = vec![];
        while TokenType::RightBrace != self.current().token_type && !self.at_end() {
            let method = self.func_declaration(FunctionType::Method)?;
            methods.push(method);
        }
        self.consume(TokenType::RightBrace, ParseErrorKind::IllegalClassDecl)?;
        Ok(Stmt::class(name, base, methods, location))
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
        Ok(Stmt::block(stmts, self.previous().location))
    }

    fn if_stmt(&mut self) -> Result<Stmt> {
        // skip the leading if token.
        let location = self.current().location;
        self.consume(TokenType::If, ParseErrorKind::FatalError)?;
        self.consume(TokenType::LeftParen, ParseErrorKind::IllegalIfStatement)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, ParseErrorKind::IllegalIfStatement)?;
        let then = self.statement()?;
        let maybe_else = self.current();
        match maybe_else.token_type {
            TokenType::Else => {
                self.step();
                let otherwise = Some(self.statement()?);
                Ok(Stmt::if_stmt(condition, then, otherwise, location))
            }
            _ => Ok(Stmt::if_stmt(condition, then, None, location)),
        }
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        // Skip "while" token.
        let location = self.current().location;
        self.consume(TokenType::While, ParseErrorKind::FatalError)?;
        self.consume(TokenType::LeftParen, ParseErrorKind::IllegalWhile)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, ParseErrorKind::IllegalWhile)?;
        let body = self.statement()?;
        Ok(Stmt::while_stmt(condition, body, location))
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        // Skip "for" token.
        let location = self.current().location;
        self.consume(TokenType::For, ParseErrorKind::FatalError)?;
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
            body = Stmt::block(vec![body, Stmt::expr(inc)], location);
        }
        let cond: Expr = condition.unwrap_or(Expr::literal(
            LoxValue::Boolean(true),
            self.current().location,
        ));
        body = Stmt::while_stmt(cond, body, location);
        if let Some(init) = initializer {
            body = Stmt::block(vec![init, body], location);
        }
        Ok(body)
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        let location = self.current().location;
        // Expect print token, otherwise we shouldn't executing this
        // method
        self.consume(TokenType::Print, ParseErrorKind::FatalError)?;
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
        Ok(Stmt::print(expr, location))
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        let location = self.current().location;
        self.consume(TokenType::Return, ParseErrorKind::FatalError)?;
        let token = self.current();
        if TokenType::SemiColon != token.token_type {
            let value = self.expression()?;
            self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
            Ok(Stmt::return_(Some(value), location))
        } else {
            self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
            Ok(Stmt::return_(None, location))
        }
    }

    fn break_stmt(&mut self) -> Result<Stmt> {
        let location = self.current().location;
        self.consume(TokenType::Break, ParseErrorKind::FatalError)?;
        self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
        Ok(Stmt::break_(location))
    }

    fn continue_stmt(&mut self) -> Result<Stmt> {
        let location = self.current().location;
        self.consume(TokenType::Continue, ParseErrorKind::FatalError)?;
        self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
        Ok(Stmt::continue_(location))
    }

    fn expression_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, ParseErrorKind::MissingSemiColon)?;
        Ok(Stmt::expr(expr))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let l_value = self.ternary()?;
        let token = self.current();
        match token.token_type {
            TokenType::Equal => {
                self.step();
                let r_value = self.assignment()?;
                match l_value.kind {
                    ExprKind::Var { name, .. } => Ok(Expr::assign(name, r_value, l_value.location)),
                    ExprKind::Get { name, object } => {
                        Ok(Expr::set(name, *object, r_value, l_value.location))
                    }
                    _ => Err(ParseError::new(
                        ParseErrorKind::RvToRvAssignment,
                        &l_value.location,
                    )),
                }
            }
            _ => Ok(l_value),
        }
    }

    fn ternary(&mut self) -> Result<Expr> {
        let location = self.current().location;
        let root = self.logical_or()?;
        let token = self.current();
        match token.token_type {
            TokenType::Qmark => {
                self.step();
                let left = self.logical_or()?;
                self.consume(TokenType::Colon, ParseErrorKind::MissingTernaryColon)?;
                let right = self.ternary()?;
                Ok(Expr::ternary(root, left, right, location))
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
                let location = self.current().location;
                self.step();
                let right = self.logical_and()?;
                Ok(Expr::logical(left, operator, right, location))
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
                let location = self.current().location;
                self.step();
                let right = self.equality()?;
                Ok(Expr::logical(left, operator, right, location))
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
                    let location = self.current().location;
                    self.step();
                    let right = self.comparison()?;
                    left = Expr::binary(left, operator, right, location);
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
                    let location = self.current().location;
                    self.step();
                    let right = self.term()?;
                    left = Expr::binary(left, operator, right, location);
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
                    let location = self.current().location;
                    self.step();
                    let right = self.factor()?;
                    left = Expr::binary(left, operator, right, location);
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
                    let location = self.current().location;
                    self.step();
                    let right = self.unary()?;
                    left = Expr::binary(left, operator, right, location);
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
                let location = self.current().location;
                self.step();
                let expr = self.unary()?;
                Ok(Expr::unary(operator, expr, location))
            }
            _ => self.lambda(),
        }
    }

    fn lambda(&mut self) -> Result<Expr> {
        let token = self.current();
        match token.token_type {
            TokenType::Fun => {
                self.step();
                let location = self.current().location;
                self.consume(TokenType::LeftParen, ParseErrorKind::IllegalFunctionDecl)?;
                let token = self.current();
                let mut params: Vec<String> = vec![];
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
                self.consume(TokenType::RightParen, ParseErrorKind::UnbalancedParentheses)?;
                // Parse definition block
                let body = self.block()?;
                Ok(Expr::lambda(params, body, location))
            }
            _ => self.call(),
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;
        loop {
            let token = self.current();
            match token.token_type {
                TokenType::LeftParen => {
                    expr = self.finish_call(expr)?;
                }
                TokenType::Dot => {
                    self.step();
                    self.consume(TokenType::Identifier, ParseErrorKind::MissingPropertyName)?;
                    let name = self.code.get_identifier(self.previous());
                    expr = Expr::get(name, expr, self.previous().location);
                }
                _ => break Ok(expr),
            };
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
                        &self.current().location,
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
        let location = callee.location;
        Ok(Expr::call_expr(callee, args, location))
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.current();
        match token.token_type {
            TokenType::False
            | TokenType::True
            | TokenType::Number
            | TokenType::Nil
            | TokenType::String => {
                let literal = Expr::literal(self.code.get_value(token), token.location);
                self.step();
                Ok(literal)
            }
            TokenType::LeftParen => {
                self.step();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, ParseErrorKind::UnbalancedParentheses)?;
                Ok(Expr::grouping(expr, self.previous().location))
            }
            TokenType::Identifier => {
                let expr = Expr::variable(self.code.get_identifier(token), token.location);
                self.step();
                Ok(expr)
            }
            TokenType::This => {
                self.step();
                Ok(Expr::this(self.previous().location))
            }
            TokenType::Super => {
                let location = token.location.clone();
                self.step();
                self.consume(TokenType::Dot, ParseErrorKind::DotExpected)?;
                self.consume(TokenType::Identifier, ParseErrorKind::MissingPropertyName)?;
                let name = self.code.get_identifier(self.previous());
                Ok(Expr::super_(name, location))
            }
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                &token.location,
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
