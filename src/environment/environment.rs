use crate::{
    utils::errors::{EnvironmentError, RoxyError},
    RoxyType,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    //TODO: Make this private
    pub values: HashMap<String, RoxyType>,
    pub enclosing: Option<Box<Environment>>,
}

// impl fmt::Display for Environment {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
//         for value in self.values.values() {

//         }

//         write!(
//             f,
//             "{:?}",
//             self.values.values().clone().collect().to_string()
//         )
//     }
// }

impl<'env> Environment {
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

    pub fn define(&mut self, name: String, value: RoxyType, globals_opt: Option<&mut Environment>) {
        if self.enclosing == None {
            if let Some(globals) = globals_opt {
                globals.values.insert(name.clone(), value.clone());
            }
        }

        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Result<RoxyType, RoxyError> {
        match self.values.get(&name) {
            Some(val) => Ok(val.to_owned()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                    return enclosing.get(name);
                }

                return Err(RoxyError::EnvironmentError(
                    EnvironmentError::UndefinedVariable(name),
                ));
            }
        }
    }

    pub fn get_at(&self, distance: usize, name: String) -> Result<RoxyType, RoxyError> {
        let env = self.ancestor(distance)?;

        match env.values.get(&name) {
            Some(value) => {
                return Ok(value.to_owned());
            }
            None => self.get(name),
        }
    }

    fn ancestor(&self, distance: usize) -> Result<Environment, RoxyError> {
        let mut env = self.clone();
        let mut i = 0;
        loop {
            if i == distance {
                break;
            }
            // Here we trust resolver has done it's work well in determining distance
            match env.enclosing {
                Some(environment) => {
                    env = *environment;
                }
                None => {
                    return Err(RoxyError::EnvironmentError(
                        EnvironmentError::EnvironmentDoesNotExistAtGivenDistance,
                    ));
                }
            }
            // env = *env.enclosing.unwrap();

            i = i + 1;
        }

        // for _i in 1..distance {
        // }

        Ok(env)
    }

    pub fn assign_at(
        &self,
        distance: usize,
        // name: Token,
        name: String,
        value: RoxyType,
    ) -> Result<(), RoxyError> {
        self.ancestor(distance)?.values.insert(name, value);
        Ok(())
    }

    pub fn assign(
        &mut self,
        name: String,
        value: RoxyType,
        globals: &mut Environment,
    ) -> Result<(), RoxyError> {
        if self.values.contains_key(&name) {
            if self.enclosing == None {
                globals.values.insert(name.clone(), value.clone());
            }

            self.values.insert(name.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.assign(name, value, globals);
        }

        return Err(RoxyError::EnvironmentError(
            EnvironmentError::UndefinedVariable(name),
        ));
    }

    // pub fn update_at(
    //     &mut self,
    //     distance: usize,
    //     name: String,
    //     value: RoxyType,
    // ) -> Result<(), RoxyError> {
    //     self.ancestor(distance)?.values.insert(name, value);
    //     Ok(())
    // }
}
