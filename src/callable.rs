use core::fmt::Debug;

use crate::{
    environment::Environment,
    interpreter::Interpreter,
    utils::errors::{InterpreterError, RoxyError},
    RoxyType, Token,
};

pub trait Callable: Debug {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<RoxyType>,
        token: Token,
        closure: Box<Environment>,
    ) -> Result<RoxyType, RoxyError>;

    fn arity(&self, token: Token) -> Result<usize, RoxyError>;
}

impl Callable for RoxyType {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<RoxyType>,
        token: Token,
        closure: Box<Environment>,
    ) -> Result<RoxyType, RoxyError> {
        match self {
            RoxyType::RoxyFunction(roxy_fn) => {
                let mut fn_env = Box::new(Environment::new_with_enclosing(closure));

                // println!("------------");
                // println!("Call: {:?}", roxy_fn.name);
                // println!("Global: {:?}", interpreter.globals);
                // println!("Current: {:?}", interpreter.environment);
                // println!("------------");

                for (i, param) in roxy_fn.params.iter().enumerate() {
                    // Here arguments vector is directly accessed because we can guarantee it won't overflow or underflow as that check is already in interpreter
                    fn_env.define(param.lexeme.clone(), arguments[i].clone());
                }

                let ret_value;
                if let Some(value) = interpreter.execute_block(roxy_fn.body.clone(), fn_env)? {
                    ret_value = value;
                } else {
                    ret_value = RoxyType::NULL;
                }

                return Ok(ret_value);
            }
            //TODO: Implement for NativeFunction
            RoxyType::NativeFunction(_) => Ok(self.to_owned()),
            _ => Err(RoxyError::InterpreterError(
                InterpreterError::CanOnlyCallFunctionsAndClasses(token),
            )),
        }
    }

    fn arity(&self, token: Token) -> Result<usize, RoxyError> {
        match self {
            //IMP-TODO: Change this
            RoxyType::RoxyFunction(roxy_fn) => Ok(roxy_fn.arity),
            RoxyType::NativeFunction(native_fn) => Ok(native_fn.arity),
            _ => Err(RoxyError::InterpreterError(
                //TODO: Do we need to make a new error over here or it's okay
                InterpreterError::CanOnlyCallFunctionsAndClasses(token),
            )),
        }
    }
}

// impl Clone for Box<dyn Callable> {
//     fn clone(&self) -> Self {
//         let a = *(*self);
//         // Box::new(().clone())
//     }
// }

// #[derive(Debug, Clone)]
// pub enum RoxyType1 {
//     Callable(Box<dyn Callable>),
// }
