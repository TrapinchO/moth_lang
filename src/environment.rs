use std::collections::HashMap;

use crate::mref::MMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Environment<T> {
    pub scopes: Vec<MMap<T>>,
}

// TODO: idk if I am happy with Option<T>, but imo it is better than a bool
// for now.
impl<T: Clone> Environment<T> {
    pub fn new(defaults: HashMap<String, T>) -> Environment<T> {
        Environment {
            scopes: vec![defaults.into()],
        }
    }

    pub fn insert(&mut self, name: &String, val: T) -> bool {
        let last_scope = self.scopes.last_mut().unwrap();
        if last_scope.read(|s| s.contains_key(name)) {
            return false;
        }
        last_scope.insert(name.clone(), val);
        true
    }

    pub fn get(&self, ident: &String) -> Option<T> {
        for scope in self.scopes.iter().rev() {
            if scope.read(|s| s.contains_key(ident)) {
                return Some(scope.get(ident).unwrap().clone());
            }
        }
        None
    }

    pub fn update(&mut self, name: &String, val: T) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.read(|s| s.contains_key(name)) {
                scope.insert(name.clone(), val);
                return true;
            }
        }
        false
    }

    pub fn contains(&self, name: &String) -> bool {
        for scope in self.scopes.iter() {
            if scope.read(|s| s.contains_key(name)) {
                return true;
            }
        }
        false
    }

    pub fn add_scope(&mut self) {
        self.add_scope_vars(HashMap::new());
    }
    pub fn add_scope_vars(&mut self, vars: HashMap<String, T>) {
        self.scopes.push(vars.into());
    }

    pub fn remove_scope(&mut self) {
        self.scopes.pop();
    }
}
