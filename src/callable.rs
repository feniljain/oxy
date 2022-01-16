use core::fmt::Debug;
use std::collections::HashMap;

use crate::{
    environment::Environment,
    interpreter::Interpreter,
    utils::errors::{InterpreterError, RoxyError},
    RoxyInstance, RoxyType, Token,
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
                interpreter.set_is_calling(true);
                if roxy_fn.is_initializer {
                    return closure.get_at(0, "this".into());
                }

                let mut fn_env;
                if roxy_fn.is_method {
                    fn_env = roxy_fn.closure.clone().unwrap();
                } else {
                    fn_env = Box::new(Environment::new_with_enclosing(closure));

                    if let Some(fn_closure) = roxy_fn.closure.clone() {
                        for (_, (name, roxy_type)) in fn_closure.values.iter().enumerate() {
                            fn_env.define(
                                name.to_owned(),
                                roxy_type.to_owned(),
                                Some(&mut interpreter.globals),
                            );
                        }
                    }
                }

                for (i, param) in roxy_fn.params.iter().enumerate() {
                    // Here arguments vector is directly accessed because we can guarantee it won't overflow or underflow as that check is already in interpreter
                    fn_env.define(
                        param.lexeme.clone(),
                        arguments[i].clone(),
                        Some(&mut interpreter.globals),
                    );
                }

                let ret_value;
                if let Some(value) = interpreter.execute_block(roxy_fn.body.clone(), fn_env)? {
                    ret_value = value;
                } else {
                    ret_value = RoxyType::NULL;
                }

                interpreter.set_is_calling(false);
                return Ok(ret_value);
            }
            //TODO: Implement for NativeFunction
            RoxyType::NativeFunction(_) => Ok(self.to_owned()),
            RoxyType::RoxyClass(roxy_class) => {
                let instance = RoxyInstance {
                    klass: roxy_class.to_owned(),
                    fields: HashMap::new(),
                };
                if let Some(initializer) = roxy_class.find_method("init".into()) {
                    RoxyType::RoxyFunction(initializer.bind(&instance)).call(
                        interpreter,
                        arguments,
                        token,
                        closure,
                    )?;
                }

                return Ok(RoxyType::RoxyInstance(instance));
            }
            _ => Err(RoxyError::InterpreterError(
                InterpreterError::CanOnlyCallFunctionsAndClasses(token),
            )),
        }
    }

    fn arity(&self, token: Token) -> Result<usize, RoxyError> {
        match self {
            //IMP-TODO: Change this
            RoxyType::RoxyFunction(roxy_fn) => Ok(roxy_fn.arity),
            RoxyType::RoxyClass(roxy_class) => {
                if let Some(initialized) = roxy_class.find_method("init".into()) {
                    return Ok(initialized.arity);
                } else {
                    return Ok(0);
                }
            }
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
