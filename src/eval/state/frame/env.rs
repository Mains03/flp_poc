use std::collections::HashMap;

use crate::cbpv::Term;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Env {
    env: HashMap<String, Term>
}

impl Env {
    pub fn new() -> Self {
        Env { env: HashMap::new() }
    }

    pub fn store(&mut self, var: &String, val: Term) {
        self.env.insert(var.clone(), val);
    }

    pub fn get_value(&self, var: &String) -> Option<Term> {
        match self.env.get(var) {
            Some(term) => Some(term.clone()),
            None => None
        }
    }
}