use std::collections::HashMap;

use crate::{
    error::Error,
    token::{Token, TokenType},
    value::{Value, ValueType},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, ValueType>>,
}

impl Environment {
    pub fn new(defaults: HashMap<String, ValueType>) -> Environment {
        Environment { scopes: vec![defaults] }
    }

    pub fn insert(&mut self, ident: &Token, val: Value) -> Result<(), Error> {
        let TokenType::Identifier(name) = &ident.val else {
            unreachable!()
        };
        let last_scope = self.scopes.last_mut().unwrap();
        if last_scope.contains_key(name) {
            return Err(Error {
                msg: format!("Name \"{}\" already exists", name),
                lines: vec![ident.loc()],
            });
        }
        last_scope.insert(name.clone(), val.val);
        Ok(())
    }

    pub fn get(&self, ident: &String, pos: (usize, usize)) -> Result<ValueType, Error> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(ident) {
                return Ok(scope.get(ident).unwrap().clone());
            }
        }
        Err(Error {
            msg: format!("Name not found: \"{}\"", ident),
            lines: vec![pos],
        })
    }

    pub fn update(&mut self, ident: &Token, val: Value) -> Result<(), Error> {
        let TokenType::Identifier(name) = &ident.val else {
            unreachable!()
        };
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                *scope.get_mut(name).unwrap() = val.val;
                return Ok(());
            }
        }
        Err(Error {
            msg: format!("Name not found: \"{}\"", name),
            lines: vec![ident.loc()],
        })
    }

    pub fn contains(&self, name: &String) -> bool {
        for scope in self.scopes.iter() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn add_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }
    pub fn add_scope_vars(&mut self, vars: HashMap<String, ValueType>) {
        self.scopes.push(vars);
    }

    pub fn remove_scope(&mut self) {
        self.scopes.pop();
    }
}
