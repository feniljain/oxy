use crate::expr::*;
use crate::tokens::TokenType;
use crate::utils::errors::{InterpreterError, RoxyError};
use crate::Interpreter;
use crate::{RoxyType, TryConversion};

impl Interpreter {
    pub fn interpret(&self, stmts: Vec<Stmt>) -> Result<(), RoxyError> {
        for stmt in stmts {
            // let value = self.evaluate(&expr)?;
            match stmt {
                Stmt::Block(_) => todo!(),
                Stmt::Class(_) => todo!(),
                Stmt::Expression(expr_stmt) => {
                    self.evaluate(&expr_stmt.expression)?;
                }
                Stmt::Function(_) => todo!(),
                Stmt::If(_) => todo!(),
                Stmt::Print(print_stmt) => {
                    let roxy_type = self.evaluate(&print_stmt.expression)?;
                    match roxy_type {
                        RoxyType::String(type_string) => println!("{:?}", type_string),
                        RoxyType::Number(number) => println!("{:?}", number),
                        RoxyType::NULL => println!("NULL"),
                        RoxyType::Boolean(type_bool) => println!("{:?}", type_bool),
                        RoxyType::Object => println!("Object"),
                    };
                }
                Stmt::VariableStmt(_) => todo!(),
                Stmt::While(_) => todo!(),
            }

            // println!("{:?}", value);
        }

        Ok(())
    }

    fn evaluate(&self, expr: &Expr) -> Result<RoxyType, RoxyError> {
        match expr {
            Expr::Assign(_) => todo!(),
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
            Expr::Call(_) => todo!(),
            Expr::Get(_) => todo!(),
            Expr::Grouping(expr) => self.evaluate(&expr.expr),
            Expr::Literal(expr) => Ok(expr.value.clone()),
            Expr::Logical(_) => todo!(),
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
            Expr::Variable(_) => todo!(),
        }
    }
}
