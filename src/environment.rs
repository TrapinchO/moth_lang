use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Environment<T> {
    pub scopes: Vec<HashMap<String, T>>,
}

impl<T: Clone> Environment<T> {
    pub fn new(defaults: HashMap<String, T>) -> Environment<T> {
        Environment { scopes: vec![defaults] }
    }

    pub fn insert(&mut self, name: &String, val: T) -> Option<()> {
        let last_scope = self.scopes.last_mut().unwrap();
        if last_scope.contains_key(name) {
            return None;
        }
        last_scope.insert(name.clone(), val);
        Some(())
    }

    pub fn get(&self, ident: &String) -> Option<T> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(ident) {
                return Some(scope.get(ident).unwrap().clone());
            }
        }
        None
    }

    pub fn update(&mut self, name: &String, val: T) -> Option<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                *scope.get_mut(name).unwrap() = val;
                return Some(());
            }
        }
        None
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
    pub fn add_scope_vars(&mut self, vars: HashMap<String, T>) {
        self.scopes.push(vars);
    }

    pub fn remove_scope(&mut self) {
        self.scopes.pop();
    }
}
