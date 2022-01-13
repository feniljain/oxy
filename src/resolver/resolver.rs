use std::collections::HashMap;

use crate::{
    expr::{Expr, Function, Stmt},
    interpreter::Interpreter,
    utils::errors::{ResolutionError, RoxyError},
    Token,
};

//TODO: When break statement is implemented add a check for them to be present only in loops,
//similar to the way we did here for return statement
//TODO: Extend the resolver to report an error if a local variable is never used.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClassType {
    None,
    Class,
    Subclass,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    //We have use this as a stack
    scopes: Vec<HashMap<String, bool>>,
    curr_func_type: FunctionType,
    curr_class_type: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            curr_func_type: FunctionType::None,
            curr_class_type: ClassType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn declare_or_define(&mut self, name: Token, is_define: bool) -> Result<(), RoxyError> {
        if !self.scopes.is_empty() {
            let len = self.scopes.len();
            let scope = match self.scopes.get_mut(len - 1) {
                Some(scope) => Ok(scope),
                None => Err(RoxyError::ResolutionError(
                    ResolutionError::InvalidScopeAccess(name.clone()),
                )),
            }?;

            if !is_define {
                if scope.contains_key(&name.lexeme) {
                    return Err(RoxyError::ResolutionError(
                        ResolutionError::AlreadyAVariableWithThisNameInThisScope(name.clone()),
                    ));
                }
            }

            scope.insert(name.lexeme, is_define);
        }

        Ok(())
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve(&mut self, stmts: Vec<Stmt>) -> Result<(), RoxyError> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }

        Ok(())
    }

    fn resolve_expr(&mut self, expr: Expr) -> Result<(), RoxyError> {
        match expr {
            Expr::Assign(assign_expr) => {
                self.resolve_expr(*assign_expr.value)?;
            }
            Expr::Binary(binary_expr) => {
                self.resolve_expr(*binary_expr.left)?;
                self.resolve_expr(*binary_expr.right)?;
            }
            Expr::Call(call_expr) => {
                self.resolve_expr(*call_expr.callee)?;
                for arg in call_expr.arguments {
                    self.resolve_expr(arg)?;
                }
            }
            Expr::Get(get_expr) => {
                self.resolve_expr(*get_expr.object)?;
            }
            Expr::Grouping(grouping_expr) => {
                self.resolve_expr(*grouping_expr.expr)?;
            }
            Expr::Literal(_) => {}
            Expr::Logical(logical_expr) => {
                self.resolve_expr(*logical_expr.left)?;
                self.resolve_expr(*logical_expr.right)?;
            }
            Expr::Set(set_expr) => {
                self.resolve_expr(*set_expr.value)?;
                self.resolve_expr(*set_expr.object)?;
            }
            Expr::Super(super_expr) => match self.curr_class_type {
                ClassType::None => {
                    return Err(RoxyError::ResolutionError(
                        ResolutionError::CantUseSuperInAClassWithNoSuperclass(super_expr.keyword),
                    ))
                }
                ClassType::Class => {
                    return Err(RoxyError::ResolutionError(
                        ResolutionError::CantUseSuperInAClassWithNoSuperclass(super_expr.keyword),
                    ))
                }
                ClassType::Subclass => {
                    self.resolve_local(Expr::Super(super_expr.clone()), super_expr.keyword)?;
                }
            },
            Expr::This(ref this_expr) => {
                if ClassType::None == self.curr_class_type {
                    return Err(RoxyError::ResolutionError(
                        ResolutionError::CantUseThisOutsideOfAClass(this_expr.keyword.clone()),
                    ));
                }

                self.resolve_local(expr.clone(), this_expr.keyword.clone())?;
            }
            Expr::Unary(unary_expr) => {
                self.resolve_expr(*unary_expr.right)?;
            }
            Expr::Variable(ref var_expr) => {
                if !self.scopes.is_empty() {
                    let len = self.scopes.len();
                    let scope_opt = &self.scopes.get(len - 1);
                    if let Some(scope) = scope_opt {
                        if let Some(initialized) = scope.get(&var_expr.name.lexeme) {
                            if !initialized {
                                return Err(RoxyError::ResolutionError(
                                    ResolutionError::CantReadLocalVariableInItsOwnInitializer(
                                        var_expr.name.clone(),
                                    ),
                                ));
                            } else {
                                self.resolve_local(expr.clone(), var_expr.name.clone())?;
                            }
                        } else {
                            self.resolve_local(expr.clone(), var_expr.name.clone())?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn resolve_local(&mut self, expr: Expr, name: Token) -> Result<(), RoxyError> {
        if !self.scopes.is_empty() {
            let len = self.scopes.len();
            let mut i = len - 1;
            loop {
                if self.scopes[i].contains_key(&name.lexeme) {
                    self.interpreter.resolve(expr, len - 1 - i);
                    return Ok(());
                }

                if i == 0 {
                    break;
                }
                i -= 1;
            }
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: Stmt) -> Result<(), RoxyError> {
        match stmt {
            Stmt::Block(block_stmt) => {
                self.begin_scope();
                self.resolve(block_stmt.statements)?;
                self.end_scope();
            }
            Stmt::Class(class_stmt) => {
                self.declare_or_define(class_stmt.name.clone(), false)?;

                self.curr_class_type = ClassType::Class;
                if let Some(superclass) = &class_stmt.superclass {
                    if class_stmt.name.lexeme.eq(&superclass.name.lexeme) {
                        return Err(RoxyError::ResolutionError(
                            ResolutionError::AClassCantInheritFromItself(superclass.name.clone()),
                        ));
                    }

                    self.begin_scope();

                    self.curr_class_type = ClassType::Subclass;
                    self.resolve_expr(Expr::Variable(superclass.clone()))?;
                    self.begin_scope();
                    if !self.scopes.is_empty() {
                        let len = self.scopes.len();
                        let scope_opt = self.scopes.get_mut(len - 1);
                        if let Some(scope) = scope_opt {
                            scope.insert("super".into(), true);
                        }
                    }
                }

                if !self.scopes.is_empty() {
                    let len = self.scopes.len();
                    let scope_opt = self.scopes.get_mut(len - 1);
                    if let Some(scope) = scope_opt {
                        scope.insert("this".into(), true);
                    }
                }
                for method in class_stmt.methods {
                    let mut declaration = FunctionType::Method;
                    if method.name.lexeme.eq("init") {
                        declaration = FunctionType::Initializer;
                    }

                    self.resolve_func(method, declaration)?;
                }
                self.declare_or_define(class_stmt.name, true)?;

                self.end_scope();

                if class_stmt.superclass.is_some() {
                    self.end_scope();
                }

                self.curr_class_type = ClassType::None;
            }
            Stmt::Expression(expr_stmt) => self.resolve_expr(expr_stmt.expression)?,
            Stmt::Function(func_stmt) => {
                self.declare_or_define(func_stmt.name.clone(), false)?;
                self.declare_or_define(func_stmt.name.clone(), true)?;
                self.resolve_func(func_stmt, FunctionType::Function)?;
            }
            Stmt::If(if_stmt) => {
                self.resolve_expr(if_stmt.condition)?;
                self.resolve_stmt(*if_stmt.then_branch)?;
                if let Some(else_branch) = if_stmt.else_branch {
                    self.resolve_stmt(*else_branch)?;
                }
            }
            Stmt::Print(print_stmt) => self.resolve_expr(print_stmt.expression)?,
            Stmt::VariableStmt(var_stmt) => {
                self.declare_or_define(var_stmt.name.clone(), false)?;
                if let Some(value) = var_stmt.value {
                    self.resolve_expr(value)?;
                }
                self.declare_or_define(var_stmt.name, true)?;
            }
            Stmt::While(while_stmt) => {
                self.resolve_expr(while_stmt.condition)?;
                self.resolve_stmt(*while_stmt.body)?;
            }
            Stmt::Return(return_stmt) => {
                if self.curr_func_type == FunctionType::None {
                    return Err(RoxyError::ResolutionError(
                        ResolutionError::CantReturnFromTopLevelCode(return_stmt.keyword),
                    ));
                }

                if let Some(value) = return_stmt.value {
                    if self.curr_func_type == FunctionType::Initializer {
                        return Err(RoxyError::ResolutionError(
                            ResolutionError::CantReturnAValueFromAnInitializer(return_stmt.keyword),
                        ));
                    }

                    self.resolve_expr(value)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_func(
        &mut self,
        func_stmt: Function,
        func_type: FunctionType,
    ) -> Result<(), RoxyError> {
        let enclosing_func_type = self.curr_func_type;
        self.curr_func_type = func_type;

        self.begin_scope();
        for param in func_stmt.params {
            self.declare_or_define(param.clone(), false)?;
            self.declare_or_define(param, true)?;
        }
        self.resolve(func_stmt.body)?;
        self.end_scope();

        self.curr_func_type = enclosing_func_type;

        Ok(())
    }
}
