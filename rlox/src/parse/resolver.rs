use super::error::{ParseError, ParseErrorKind};
use super::Result;
use super::{Expr, ExprKind, Stmt, StmtKind};
use crate::code::CodeLocation;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum IdentifierType {
    NotSet,
    Variable,
    Function,
    Class,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, IdentifierType>>,
    loop_depth: usize,
    function_depth: usize,
    class_depth: usize,
    in_subclass: bool,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            loop_depth: 0,
            function_depth: 0,
            class_depth: 0,
            in_subclass: false,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_stmts(&mut self, stmts: &mut [Stmt]) -> Vec<Result<()>> {
        stmts
            .into_iter()
            .map(|stmt| self.resolve_stmt(stmt))
            .collect()
    }

    fn resolve_stmt(&mut self, stmt: &mut Stmt) -> Result<()> {
        match &mut stmt.kind {
            StmtKind::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.resolve_stmt(stmt)?;
                }
                self.end_scope();
            }

            StmtKind::Function { name, params, body } => {
                self.function_depth += 1;
                self.declare(name, &stmt.location)?;
                self.define(name, IdentifierType::Function, &stmt.location)?;
                // Insert a new scope and define function
                // parameters in it
                self.begin_scope();
                for param in params.iter() {
                    self.declare(param, &stmt.location)?;
                    self.define(param, IdentifierType::Variable, &stmt.location)?;
                }
                self.resolve_stmt(body)?;
                self.end_scope();
                self.function_depth -= 1;
            }

            StmtKind::Class {
                name: class_name,
                base,
                methods,
            } => {
                self.class_depth += 1;
                self.declare(class_name, &stmt.location)?;
                self.define(class_name, IdentifierType::Class, &stmt.location)?;
                // If we're in a subclass
                //   - set the flag is_subclass = true
                //   - set resolution depth for superclass
                //   - set insert a new scope and define "super" in it
                if let Some(base_expr) = base {
                    self.in_subclass = true;
                    match &base_expr.kind {
                        ExprKind::Var { name, depth } if name != class_name => {
                            let identifier_type = self.get_type(name, *depth);
                            if identifier_type != &IdentifierType::Class {
                                return Err(ParseError::new(
                                    ParseErrorKind::BaseClassNotAClass,
                                    &base_expr.location,
                                ));
                            }
                            self.resolve_expr(base_expr)?
                        }
                        _ => {
                            return Err(ParseError::new(
                                ParseErrorKind::IllegalClassDecl,
                                &base_expr.location,
                            ))
                        }
                    }
                    self.begin_scope();
                    self.declare("super", &stmt.location)?;
                    self.define("super", IdentifierType::Variable, &stmt.location)?;
                }
                // insert a new scope and define "this" in it
                self.begin_scope();
                self.declare("this", &stmt.location)?;
                self.define("this", IdentifierType::Variable, &stmt.location)?;
                for method in methods {
                    self.resolve_stmt(method)?;
                }
                self.end_scope();
                // if we're in a subclass exit the extra
                // environment we added
                if base.is_some() {
                    self.end_scope();
                    self.in_subclass = false;
                }
                self.class_depth -= 1;
            }

            StmtKind::Var { name, init } => {
                self.declare(name, &stmt.location)?;
                match init {
                    Some(expr) => {
                        self.resolve_expr(expr)?;
                    }
                    _ => {}
                };
                self.define(name, IdentifierType::Variable, &stmt.location)?;
            }

            StmtKind::Expr(expr) => {
                self.resolve_expr(expr)?;
            }

            StmtKind::If {
                condition,
                then,
                otherwise,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then)?;
                if let Some(else_body) = otherwise {
                    self.resolve_stmt(else_body)?;
                }
            }

            StmtKind::While { condition, body } => {
                self.loop_depth += 1;
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
                self.loop_depth -= 1;
            }

            StmtKind::Print(expr) => {
                self.resolve_expr(expr)?;
            }

            // make sure return statement is inside a function
            StmtKind::Return(_) if self.function_depth == 0 => {
                return Err(ParseError::new(
                    ParseErrorKind::ReturnAtTopLevel,
                    &stmt.location,
                ))
            }

            StmtKind::Return(Some(expr)) => {
                self.resolve_expr(expr)?;
            }

            StmtKind::Return(_) => {}

            // make sure break is inside a loop
            StmtKind::Break if self.loop_depth == 0 => {
                return Err(ParseError::new(ParseErrorKind::NotInALoop, &stmt.location))
            }
            StmtKind::Break => {}

            // make sure continue is inside a loop
            StmtKind::Continue if self.loop_depth == 0 => {
                return Err(ParseError::new(ParseErrorKind::NotInALoop, &stmt.location))
            }
            StmtKind::Continue => {}
        };
        Ok(())
    }

    fn resolve_expr(&mut self, expr_in: &mut Expr) -> Result<()> {
        let resolution: Option<String> = match &mut expr_in.kind {
            ExprKind::Literal { .. } => None,

            ExprKind::Unary { expr, .. } => {
                self.resolve_expr(expr)?;
                None
            }

            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                None
            }

            ExprKind::Logical { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                None
            }

            ExprKind::Ternary { root, left, right } => {
                self.resolve_expr(root)?;
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                None
            }

            ExprKind::Grouping { expr } => {
                self.resolve_expr(expr)?;
                None
            }

            ExprKind::Var { ref name, .. } => {
                let scope = &self.scopes[self.scopes.len() - 1];
                if let Some(IdentifierType::NotSet) = scope.get(name) {
                    return Err(ParseError::new(
                        ParseErrorKind::RecursiveInitializer,
                        &expr_in.location,
                    ));
                } else {
                    Some(name.to_owned())
                }
            }

            ExprKind::Assign { ref name, expr, .. } => {
                self.resolve_expr(expr)?;
                Some(name.to_owned())
            }

            ExprKind::Lambda { params, body } => {
                self.function_depth += 1;
                self.begin_scope();
                for param in params.iter() {
                    self.declare(param, &expr_in.location)?;
                    self.define(param, IdentifierType::Variable, &expr_in.location)?;
                }
                self.resolve_stmt(body)?;
                self.end_scope();
                self.function_depth -= 1;
                None
            }

            ExprKind::Call { callee, args } => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                None
            }

            ExprKind::Get { object, .. } => {
                self.resolve_expr(object)?;
                None
            }

            ExprKind::Set { object, value, .. } => {
                self.resolve_expr(value)?;
                self.resolve_expr(object)?;
                None
            }

            ExprKind::This { .. } | ExprKind::Super { .. } if self.class_depth == 0 => {
                return Err(ParseError::new(
                    ParseErrorKind::ThisOutsideClass,
                    &expr_in.location,
                ))
            }

            ExprKind::This { .. } => Some("this".to_owned()),

            ExprKind::Super { .. } if !self.in_subclass => {
                return Err(ParseError::new(
                    ParseErrorKind::NotASubClass,
                    &expr_in.location,
                ))
            }

            ExprKind::Super { .. } => Some("super".to_owned()),
        };
        if let Some(name) = resolution {
            self.resolve_local(expr_in, &name)?;
        }
        Ok(())
    }

    fn resolve_local(&mut self, expr: &mut Expr, name: &str) -> Result<()> {
        // set resolution depth for given expression
        let mut cursor = self.scopes.len() as i32 - 1;
        while cursor >= 0 {
            if self.scopes[cursor as usize].contains_key(name) {
                let depth = self.scopes.len() - 1 - cursor as usize;
                expr.set_depth(depth);
                break;
            }
            cursor -= 1;
        }
        Ok(())
    }

    fn declare(&mut self, key: &str, location: &CodeLocation) -> Result<()> {
        let nscopes = self.scopes.len();
        let scope = &mut self.scopes[nscopes - 1];
        if scope.contains_key(key) {
            Err(ParseError::new(
                ParseErrorKind::AlreadyDeclaredIdentifier,
                location,
            ))
        } else {
            scope.insert(key.to_owned(), IdentifierType::NotSet);
            Ok(())
        }
    }

    fn define(
        &mut self,
        key: &str,
        id_type: IdentifierType,
        _location: &CodeLocation,
    ) -> Result<()> {
        let nscopes = self.scopes.len();
        let scope = &mut self.scopes[nscopes - 1];
        scope.insert(key.to_owned(), id_type);
        Ok(())
    }

    fn get_type(&self, key: &str, depth: usize) -> &IdentifierType {
        let cursor = (self.scopes.len() as i32) - (depth as i32) -1;
        if self.scopes[cursor as usize].contains_key(key) {
            self.scopes[cursor as usize].get(key).unwrap()
        } else {
            panic!("Identifier {} not in scope", key);
        }
    }
}
