use crate::{
    utils::errors::{EnvironmentError, RoxyError},
    RoxyType, Token,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, RoxyType>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_with_enclosing(enclosing: Box<Environment>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: RoxyType) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: Token) -> Result<RoxyType, RoxyError> {
        match self.values.get(&name.lexeme) {
            Some(val) => Ok(val.to_owned()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                    return enclosing.get(name);
                }

                return Err(RoxyError::EnvironmentError(
                    EnvironmentError::UndefinedVariable(name.lexeme),
                ));
            }
        }
    }

    pub fn assign(&mut self, name: Token, value: RoxyType) -> Result<(), RoxyError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.assign(name, value);
        }

        return Err(RoxyError::EnvironmentError(
            EnvironmentError::UndefinedVariable(name.lexeme),
        ));
    }
}
