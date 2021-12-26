use crate::environment::Environment;
use crate::expr::*;
use crate::tokens::TokenType;
use crate::utils::errors::{InterpreterError, RoxyError};
use crate::{RoxyType, TryConversion};

pub struct Interpreter {
    environment: Box<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Box::new(Environment::new()),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), RoxyError> {
        for stmt in stmts {
            match stmt {
                Stmt::Block(block) => {
                    self.execute_block(block.statements, None)?;
                }
                Stmt::Class(_) => todo!(),
                Stmt::Expression(expr_stmt) => {
                    self.evaluate(&expr_stmt.expression)?;
                }
                Stmt::Function(_) => todo!(),
                Stmt::If(if_stmt) => {
                    let value = self.evaluate(&if_stmt.condition)?;
                    if self.is_truthy(&value) {
                        self.interpret(vec![*if_stmt.then_branch])?;
                    } else {
                        if let Some(else_branch) = if_stmt.else_branch {
                            self.interpret(vec![*else_branch])?;
                        }
                    }

                    return Ok(());
                }
                Stmt::Print(print_stmt) => {
                    //TODO: Define print and println properly
                    let roxy_type = self.evaluate(&print_stmt.expression)?;
                    match roxy_type {
                        RoxyType::String(type_string) => println!("{:?}", type_string),
                        RoxyType::Number(number) => println!("{:?}", number),
                        RoxyType::NULL => println!("NULL"),
                        RoxyType::Boolean(type_bool) => println!("{:?}", type_bool),
                        RoxyType::Object => println!("Object"),
                    };
                }
                Stmt::VariableStmt(var_stmt) => {
                    let mut value: RoxyType = RoxyType::NULL;
                    if let Some(expr) = var_stmt.value {
                        value = self.evaluate(&expr)?;
                    }

                    self.environment.define(var_stmt.name.lexeme, value);
                }
                Stmt::While(while_stmt) => {
                    let mut condition = self.evaluate(&while_stmt.condition)?;
                    //TODO: Implement break(and continue) keywords
                    while self.is_truthy(&condition) {
                        self.interpret(vec![*(while_stmt.body.clone())])?;
                        condition = self.evaluate(&while_stmt.condition)?;
                    }
                }
            }
        }

        Ok(())
    }

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
                    TokenType::Plus => {
                        // println!("lEft: {:?} \nRight: {:?}\n--", left, right);
                        match (left, right) {
                            (RoxyType::String(val_left), RoxyType::String(val_right)) => {
                                Ok(RoxyType::String(format!("{}{}", val_left, val_right)))
                            }
                            (RoxyType::Number(val_left), RoxyType::Number(val_right)) => {
                                Ok(RoxyType::Number(val_left + val_right))
                            }
                            _ => Err(RoxyError::InterpreterError(
                                InterpreterError::InvalidOperationOnGivenTypes(
                                    expr.operator.clone(),
                                ),
                            )),
                        }
                    }
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
            Expr::Call(_) => todo!(),
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

    fn execute_block(
        &mut self,
        stmts: Vec<Stmt>,
        env: Option<Environment>,
    ) -> Result<(), RoxyError> {
        let new = match env {
            Some(e) => Box::new(Environment::new_with_enclosing(Box::new(e))),
            None => Box::new(Environment::new_with_enclosing(self.environment.clone())),
        };
        self.environment = new;
        let res = self.interpret(stmts);
        self.environment = self.environment.clone().enclosing.unwrap();
        res
    }

    fn is_truthy(&self, value: &RoxyType) -> bool {
        match value {
            RoxyType::Boolean(val) => val.clone(),
            RoxyType::String(_) => true,
            _ => false,
        }
    }
}
