use std::collections::HashMap;

use crate::cbpv::Term;

#[derive(Clone, Debug)]
pub struct Env {
    env: HashMap<String, Term>
}

impl Env {
    pub fn new() -> Self {
        Env { env: HashMap::new() }
    }

    pub fn store(&mut self, var: &String, val: Term) {
        todo!()
    }

    pub fn get_value(&self, var: &String) -> Option<Term> {
        todo!()
    }
}