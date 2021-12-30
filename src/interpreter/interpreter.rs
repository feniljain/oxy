use crate::environment::Environment;
use crate::tokens::TokenType;
use crate::utils::errors::{InternalError, InterpreterError, RoxyError};
use crate::{callable::Callable, expr::*, NativeFunction, RoxyFunction, RoxyType, TryConversion};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Interpreter {
    pub globals: Box<Environment>,
    //TODO: Make this private
    pub environment: Box<Environment>,
}

impl<'a> Interpreter {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<RoxyType, RoxyError> {
        match expr {
            Expr::Assign(expr) => {
                let value = self.evaluate(&expr.value)?;
                self.environment.assign(expr.name.clone(), value.clone())?;
                return Ok(value);
            }
            Expr::Binary(expr) => {
                let left = self.evaluate(&expr.left)?;
                let right = self.evaluate(&expr.right)?;
                match expr.operator.token_type {
                    TokenType::Minus => Ok(RoxyType::Number(
                        (f64::try_conversion(left, expr.operator.clone())?)
                            - (f64::try_conversion(right, expr.operator.clone())?),
                    )),
                    TokenType::Slash => {
                        let val_left = f64::try_conversion(left, expr.operator.clone())?;
                        let val_right = f64::try_conversion(right, expr.operator.clone())?;

                        if val_right == 0.0 {
                            return Err(RoxyError::InterpreterError(
                                InterpreterError::DivideByZeroError(expr.operator.clone()),
                            ));
                        }

                        Ok(RoxyType::Number(val_left / val_right))
                    }
                    TokenType::Star => Ok(RoxyType::Number(
                        (f64::try_conversion(left, expr.operator.clone())?)
                            * (f64::try_conversion(right, expr.operator.clone())?),
                    )),
                    TokenType::Plus => match (left, right) {
                        (RoxyType::String(val_left), RoxyType::String(val_right)) => {
                            Ok(RoxyType::String(format!("{}{}", val_left, val_right)))
                        }
                        (RoxyType::Number(val_left), RoxyType::Number(val_right)) => {
                            Ok(RoxyType::Number(val_left + val_right))
                        }
                        _ => Err(RoxyError::InterpreterError(
                            InterpreterError::InvalidOperationOnGivenTypes(expr.operator.clone()),
                        )),
                    },
                    TokenType::Greater => Ok(RoxyType::Boolean(
                        (f64::try_conversion(left, expr.operator.clone())?)
                            > (f64::try_conversion(right, expr.operator.clone())?),
                    )),
                    TokenType::GreaterEqual => Ok(RoxyType::Boolean(
                        (f64::try_conversion(left, expr.operator.clone())?)
                            >= (f64::try_conversion(right, expr.operator.clone())?),
                    )),
                    TokenType::Less => Ok(RoxyType::Boolean(
                        (f64::try_conversion(left, expr.operator.clone())?)
                            < (f64::try_conversion(right, expr.operator.clone())?),
                    )),
                    TokenType::LessEqual => Ok(RoxyType::Boolean(
                        (f64::try_conversion(left, expr.operator.clone())?)
                            <= (f64::try_conversion(right, expr.operator.clone())?),
                    )),
                    TokenType::BangEqual => Ok(RoxyType::Boolean(left != right)),
                    TokenType::EqualEqual => Ok(RoxyType::Boolean(left == right)),
                    _ => Err(RoxyError::InterpreterError(
                        InterpreterError::InvalidOperationOnGivenTypes(expr.operator.clone()),
                    )),
                }
            }
            Expr::Call(expr) => {
                let callee = self.evaluate(&expr.callee)?;

                let mut arguments: Vec<RoxyType> = vec![];
                for argument in &expr.arguments {
                    arguments.push(self.evaluate(&argument.to_owned())?);
                }

                if arguments.len() != callee.arity(expr.paren.clone())? {
                    return Err(RoxyError::InterpreterError(
                        InterpreterError::ExpectednArgsGotmArgs(
                            arguments.len(),
                            callee.arity(expr.paren.clone())?,
                            expr.paren.clone(),
                        ),
                    ));
                }
                return callee.call(
                    self,
                    arguments,
                    expr.paren.clone(),
                    self.environment.clone(),
                );
            }
            Expr::Get(_) => todo!(),
            Expr::Grouping(expr) => self.evaluate(&expr.expr),
            Expr::Literal(expr) => Ok(expr.value.clone()),
            Expr::Logical(expr) => {
                let left = self.evaluate(&expr.left)?;
                if expr.operator.token_type == TokenType::Or {
                    if self.is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !self.is_truthy(&left) {
                        return Ok(left);
                    }
                }

                return Ok(self.evaluate(&expr.right)?);
            }
            Expr::Set(_) => todo!(),
            Expr::Super(_) => todo!(),
            Expr::This(_) => todo!(),
            Expr::Unary(expr) => {
                let right = self.evaluate(&expr.right)?;
                match expr.operator.token_type {
                    TokenType::Minus => Ok(RoxyType::Number(
                        -(f64::try_conversion(right, expr.operator.clone())?),
                    )),
                    TokenType::Bang => Ok(RoxyType::Boolean(
                        !(bool::try_conversion(right, expr.operator.clone())?),
                    )),
                    _ => Err(RoxyError::InterpreterError(
                        InterpreterError::InvalidUnaryOperator(expr.operator.clone()),
                    )),
                }
            }
            Expr::Variable(variable) => {
                return self.environment.get(variable.name.clone());
            }
        }
    }

    pub fn execute_block(
        &mut self,
        stmts: Vec<Stmt>,
        env: Box<Environment>,
    ) -> Result<Option<RoxyType>, RoxyError> {
        // let new = match env {
        //     // Some(e) => Box::new(Environment::new_with_enclosing(Box::new(e))),
        //     Some(e) => Box::new(e),
        //     None => Box::new(Environment::new_with_enclosing(self.environment.clone())),
        // };

        self.environment = env;

        // if let Some(env) = env_opt {
        //     new_env = Environment::new_with_enclosing(env);
        // } else {
        //     new_env = Environment::new();
        // }

        //         self.environment = env_opt;
        for stmt in stmts {
            if let Some(value) = self.interpret(stmt)? {
                return Ok(Some(value));
            }
        }

        //TODO: Think how to remove this unwrap
        // self.environment = self.environment.clone().enclosing.unwrap();
        self.environment = self.environment.clone().enclosing.unwrap();

        Ok(None)
    }

    pub fn interpret(&mut self, stmt: Stmt) -> Result<Option<RoxyType>, RoxyError> {
        match stmt {
            Stmt::Block(block) => {
                let block_env = Box::new(Environment::new_with_enclosing(self.environment.clone()));
                self.execute_block(block.statements, block_env)?;
            }
            Stmt::Class(_) => todo!(),
            Stmt::Expression(expr_stmt) => {
                self.evaluate(&expr_stmt.expression)?;
            }
            Stmt::Function(function) => {
                self.environment.define(
                    function.name.lexeme.clone(),
                    RoxyType::RoxyFunction(RoxyFunction {
                        name: function.name.lexeme.clone(),
                        arity: function.params.len(),
                        params: function.params.clone(),
                        body: function.body.clone(),
                        closure: self.environment.clone(),
                    }),
                );
                if let None = self.environment.enclosing {
                    self.globals.define(
                        function.name.lexeme.clone(),
                        RoxyType::RoxyFunction(RoxyFunction {
                            name: function.name.lexeme.clone(),
                            arity: function.params.len(),
                            params: function.params,
                            body: function.body,
                            closure: self.environment.clone(),
                        }),
                    );
                }

                // println!("============");
                // println!("Call: {:?}", function.name.lexeme);
                // println!("Global: {:?}", self.globals);
                // println!("Current: {:?}", self.environment);
                // println!("============");
            }
            Stmt::If(if_stmt) => {
                let value = self.evaluate(&if_stmt.condition)?;
                if self.is_truthy(&value) {
                    self.interpret(*if_stmt.then_branch)?;
                } else {
                    if let Some(else_branch) = if_stmt.else_branch {
                        self.interpret(*else_branch)?;
                    }
                }
            }
            Stmt::Print(print_stmt) => {
                //TODO: Implement print and println properly
                let roxy_type = self.evaluate(&print_stmt.expression)?;
                println!("{:?}", roxy_type.to_string());
                //NOTE: In testing
                // match roxy_type {
                //     RoxyType::String(type_string) => println!("{:?}", type_string),
                //     RoxyType::Number(number) => println!("{:?}", number),
                //     RoxyType::NULL => println!("NULL"),
                //     RoxyType::Boolean(type_bool) => println!("{:?}", type_bool),
                //     RoxyType::Object => println!("Object"),
                //     RoxyType::RoxyFunction => println!("RoxyFunction"),
                // };
            }
            Stmt::VariableStmt(var_stmt) => {
                let mut value: RoxyType = RoxyType::NULL;
                if let Some(expr) = var_stmt.value {
                    value = self.evaluate(&expr)?;
                }

                self.environment
                    .define(var_stmt.name.lexeme.clone(), value.clone());
                if let None = self.environment.enclosing {
                    self.globals.define(var_stmt.name.lexeme, value);
                }
            }
            Stmt::While(while_stmt) => {
                let mut condition = self.evaluate(&while_stmt.condition)?;
                //TODO: Implement break(and continue) keywords
                while self.is_truthy(&condition) {
                    self.interpret(*(while_stmt.body.clone()))?;
                    condition = self.evaluate(&while_stmt.condition)?;
                }
            }
            Stmt::Return(return_stmt) => {
                if let Some(expr) = return_stmt.value {
                    //TODO: HOW TO RETURN THIS value?
                    let value = self.evaluate(&expr)?;
                    return Ok(Some(value));
                }
            }
        }

        Ok(None)
    }

    fn is_truthy(&self, value: &RoxyType) -> bool {
        match value {
            RoxyType::Boolean(val) => val.clone(),
            RoxyType::String(_) => true,
            _ => false,
        }
    }

    pub fn new() -> Self {
        //NOTE: Look into global env too while resolving the function calls
        let mut global_env = Environment::new();
        global_env.define(
            String::from("clock"),
            RoxyType::NativeFunction(NativeFunction {
                name: String::from("check"),
                params: vec![],
                arity: 0,
                callable: |_: &RoxyType,
                           _: &mut Interpreter,
                           _: Vec<RoxyType>,
                           token: Token|
                 -> Result<RoxyType, RoxyError> {
                    let start = SystemTime::now();
                    match start.duration_since(UNIX_EPOCH) {
                        Ok(since_the_epoch) => {
                            Ok(RoxyType::Number(since_the_epoch.as_millis() as f64))
                        }
                        Err(_) => Err(RoxyError::InternalError(
                            InternalError::TimeConversionError(token),
                        )),
                    }
                },
            }),
        );

        return Self {
            environment: Box::new(global_env.clone()),
            globals: Box::new(global_env),
        };
    }
}
