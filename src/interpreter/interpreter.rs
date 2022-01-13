use crate::environment::Environment;
use crate::tokens::TokenType;
use crate::utils::errors::{InternalError, InterpreterError, RoxyError};
use crate::RoxyClass;
use crate::{callable::Callable, expr::*, NativeFunction, RoxyFunction, RoxyType, TryConversion};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub globals: Box<Environment>,
    //TODO: Make this private
    pub environment: Box<Environment>,
    locals: HashMap<String, usize>,
    inside_function: bool,
}

impl Interpreter {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<RoxyType, RoxyError> {
        match expr {
            Expr::Assign(assign_expr) => {
                let value = self.evaluate(&assign_expr.value)?;
                // let distance_opt = self.locals.get(&expr.to_string());
                // if let Some(distance) = distance_opt {
                //     self.environment.assign_at(
                //         *distance,
                //         assign_expr.name.lexeme.clone(),
                //         value.clone(),
                //     )?;
                // } else {
                //     self.globals
                //         .assign(assign_expr.name.lexeme.clone(), value.clone())?;
                // }

                self.environment.assign(
                    assign_expr.name.lexeme.clone(),
                    value.clone(),
                    &mut self.globals,
                )?;

                // if let None = self.environment.enclosing {
                //     self.globals
                //         .assign(assign_expr.name.lexeme.clone(), value.clone())?;
                // }

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
            Expr::Get(get_expr) => {
                let object = self.evaluate(&get_expr.object)?;
                if let RoxyType::RoxyInstance(mut roxy_instance) = object {
                    return roxy_instance.get(get_expr.name.clone());
                }

                return Err(RoxyError::InterpreterError(
                    InterpreterError::OnlyInstancesHaveKeyword(
                        "properties".into(),
                        get_expr.name.clone(),
                    ),
                ));
            }
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
            Expr::Set(set_expr) => {
                let object = self.evaluate(&set_expr.object)?;

                if let RoxyType::RoxyInstance(mut roxy_instance) = object {
                    let value = self.evaluate(&set_expr.value)?;
                    roxy_instance.set(set_expr.name.clone(), value.clone());
                    for (key, value) in self.environment.clone().values {
                        if let RoxyType::RoxyInstance(instance) = value {
                            if instance.klass.name == roxy_instance.clone().klass.name {
                                self.environment.assign(
                                    key.to_owned(),
                                    RoxyType::RoxyInstance(roxy_instance.clone()),
                                    &mut self.globals,
                                )?;
                            }
                        }
                    }

                    self.environment.assign(
                        roxy_instance.klass.name.clone(),
                        RoxyType::RoxyInstance(roxy_instance.clone()),
                        &mut self.globals,
                    )?;
                    // if let None = self.environment.enclosing {
                    //     for (key, value) in self.globals.clone().values {
                    //         if let RoxyType::RoxyInstance(instance) = value {
                    //             if instance.klass.name == roxy_instance.clone().klass.name {
                    //                 self.globals.assign(
                    //                     key.to_owned(),
                    //                     RoxyType::RoxyInstance(roxy_instance.clone()),
                    //                 )?;
                    //             }
                    //         }
                    //     }
                    //     self.globals.assign(
                    //         roxy_instance.klass.name.clone(),
                    //         RoxyType::RoxyInstance(roxy_instance),
                    //     )?;
                    // }

                    return Ok(value);
                }

                return Err(RoxyError::InterpreterError(
                    InterpreterError::OnlyInstancesHaveKeyword(
                        "fields".into(),
                        set_expr.name.clone(),
                    ),
                ));
            }
            Expr::Super(super_expr) => {
                let distance = self.locals[&Expr::Super(super_expr.to_owned()).to_string()];
                let superclass = self.environment.get_at(distance, "super".into())?;
                let object = self.environment.get_at(distance - 1, "this".into())?;

                if let RoxyType::RoxyClass(superclass) = superclass {
                    if let Some(method) = superclass.find_method(super_expr.method.lexeme.clone()) {
                        match object {
                            RoxyType::RoxyInstance(instance) => {
                                return Ok(RoxyType::RoxyFunction(method.bind(&instance)));
                            }
                            _ => unreachable!(),
                        }
                    }

                    return Err(RoxyError::InterpreterError(
                        InterpreterError::UndefinedProperty(super_expr.method.clone()),
                    ));
                }

                return Err(RoxyError::InterpreterError(
                    InterpreterError::ExpectedRoxyClass,
                ));
            }
            Expr::This(this_expr) => {
                return self.environment.get(this_expr.keyword.lexeme.clone());
                // return self.look_up_variable(this_expr.keyword.clone(), expr.clone());
            }
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
                let res = self.look_up_variable(variable.name.clone(), expr.to_owned());
                return res;
                // return self.environment.get(variable.name.clone());
            }
        }
    }

    fn look_up_variable(&mut self, name: Token, variable: Expr) -> Result<RoxyType, RoxyError> {
        let distance_opt = self.locals.get(&variable.to_string());

        if let Some(distance) = distance_opt {
            let res = self.environment.get_at(*distance, name.lexeme)?;
            return Ok(res);
        } else {
            let res = self.globals.get(name.lexeme);
            return res;
        }
    }

    pub fn execute_block(
        &mut self,
        stmts: Vec<Stmt>,
        env: Box<Environment>,
    ) -> Result<Option<RoxyType>, RoxyError> {
        self.environment = env;
        // println!("Env at start: {:?}", self.environment);

        for stmt in stmts.clone() {
            //TODO: Remove clone from here
            if let Some(value) = self.interpret(stmt.clone())? {
                // println!("Stmt: {:?}", stmt);
                // println!("Environment: {:?}", self.environment.enclosing);
                self.environment = self.environment.clone().enclosing.unwrap();
                return Ok(Some(value));
            }
            println!("Stmt: {:?}", stmt);
            println!("Environment: {:?}", self.environment.enclosing);
        }

        // println!("Env at end: {:?}", self.environment);
        //TODO: Think how to remove this unwrap
        self.environment = self.environment.clone().enclosing.unwrap();

        Ok(None)
    }

    pub fn interpret(&mut self, stmt: Stmt) -> Result<Option<RoxyType>, RoxyError> {
        match stmt {
            Stmt::Block(block) => {
                let block_env = Box::new(Environment::new_with_enclosing(self.environment.clone()));
                return self.execute_block(block.statements, block_env);
            }
            Stmt::Class(class_stmt) => {
                let mut superclass_opt = None;

                self.environment.define(
                    class_stmt.name.lexeme.clone(),
                    RoxyType::NULL,
                    Some(&mut self.globals),
                );

                if let Some(superclass) = &class_stmt.superclass {
                    let roxy_type = self.evaluate(&Expr::Variable(superclass.clone()))?;
                    match roxy_type {
                        RoxyType::RoxyClass(class) => {
                            let mut superclass_env =
                                Environment::new_with_enclosing(self.environment.clone());
                            superclass_env.define(
                                "super".into(),
                                RoxyType::RoxyClass(class.clone()),
                                Some(&mut self.globals),
                            );

                            self.environment = Box::new(superclass_env);

                            superclass_opt = Some(Box::new(class));
                        }
                        _ => {
                            return Err(RoxyError::InterpreterError(
                                InterpreterError::SuperclassMustBeAClass(superclass.name.clone()),
                            ));
                        }
                    }
                }

                // if let None = self.environment.enclosing {
                //     self.globals
                //         .define(class_stmt.name.lexeme.clone(), RoxyType::NULL);
                // }

                let mut methods = HashMap::new();
                for method in class_stmt.methods {
                    methods.insert(
                        method.name.lexeme.clone(),
                        RoxyFunction {
                            name: method.name.lexeme.clone(),
                            arity: method.params.len(),
                            params: method.params,
                            body: method.body,
                            closure: Some(self.environment.clone()),
                            is_method: true,
                            is_initializer: (method.name.lexeme.eq("into")),
                        },
                    );
                }

                let klass = RoxyType::RoxyClass(RoxyClass {
                    name: class_stmt.name.lexeme.clone(),
                    methods,
                    superclass: superclass_opt,
                });

                if class_stmt.superclass.is_some() {
                    self.environment = self.environment.enclosing.clone().unwrap();
                }

                self.environment.assign(
                    class_stmt.name.lexeme.clone(),
                    klass.clone(),
                    &mut self.globals,
                )?;

                // if let None = self.environment.enclosing {
                //     self.globals.assign(class_stmt.name.lexeme, klass)?;
                // }
            }
            Stmt::Expression(expr_stmt) => {
                self.evaluate(&expr_stmt.expression)?;
            }
            // TODO: Implement anonymous functions
            Stmt::Function(function) => {
                let closure;
                if self.inside_function {
                    closure = Some(self.environment.clone());
                } else {
                    closure = None;
                }

                self.environment.define(
                    function.name.lexeme.clone(),
                    RoxyType::RoxyFunction(RoxyFunction {
                        name: function.name.lexeme.clone(),
                        arity: function.params.len(),
                        params: function.params.clone(),
                        body: function.body.clone(),
                        // closure: Some(self.environment.clone()),
                        closure,
                        is_method: false,
                        is_initializer: false,
                    }),
                    Some(&mut self.globals),
                );
                // if let None = self.environment.enclosing {
                //     self.globals.define(
                //         function.name.lexeme.clone(),
                //         RoxyType::RoxyFunction(RoxyFunction {
                //             name: function.name.lexeme.clone(),
                //             arity: function.params.len(),
                //             params: function.params,
                //             body: function.body,
                //             closure: self.environment.clone(),
                //             is_method: false,
                //         }),
                //     );
                // }
            }
            Stmt::If(if_stmt) => {
                let value = self.evaluate(&if_stmt.condition)?;
                if self.is_truthy(&value) {
                    return self.interpret(*if_stmt.then_branch);
                } else {
                    if let Some(else_branch) = if_stmt.else_branch {
                        return self.interpret(*else_branch);
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

                self.environment.define(
                    var_stmt.name.lexeme.clone(),
                    value.clone(),
                    Some(&mut self.globals),
                );

                // if let None = self.environment.enclosing {
                //     self.globals.define(var_stmt.name.lexeme, value);
                // }
            }
            Stmt::While(while_stmt) => {
                let mut condition = self.evaluate(&while_stmt.condition)?;
                //TODO: Implement break(and continue) keywords
                let mut i = 0;
                while self.is_truthy(&condition) {
                    if i > 10 {
                        break;
                    }
                    if let Some(value) = self.interpret(*(while_stmt.body.clone()))? {
                        return Ok(Some(value));
                    }

                    condition = self.evaluate(&while_stmt.condition)?;

                    i = i + 1;
                }
            }
            Stmt::Return(return_stmt) => {
                if let Some(expr) = return_stmt.value {
                    //TODO: HOW TO RETURN THIS value?
                    let value = self.evaluate(&expr)?;
                    return Ok(Some(value));
                }

                return Ok(Some(RoxyType::NULL));
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
            None,
        );

        return Self {
            environment: Box::new(global_env.clone()),
            globals: Box::new(global_env),
            locals: HashMap::new(),
            inside_function: false,
        };
    }

    pub fn resolve(&mut self, expr: Expr, depth: usize) {
        // let env_string =
        self.locals.insert(expr.to_string(), depth);
    }

    pub fn set_is_calling(&mut self, inside_function: bool) {
        self.inside_function = inside_function;
    }
}

//closure.rx
//super_kw.rx
