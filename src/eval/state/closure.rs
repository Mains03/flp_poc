use std::collections::HashMap;

use crate::cbpv::Term;

use super::env::env_value::EnvValue;

#[derive(Clone, Debug)]
pub struct Closure {
    pub term: Term,
    pub vars: ClosureVars
}

#[derive(Clone, Debug)]
pub struct ClosureVars {
    vars: HashMap<String, EnvValue>
}

impl ClosureVars {
    pub fn new() -> Self {
        ClosureVars { vars: HashMap::new() }
    }

    pub fn store(&mut self, var: String, val: EnvValue) {
        self.vars.insert(var, val);
    }

    pub fn lookup(&self, var: &String) -> Option<EnvValue> {
        match self.vars.get(var) {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}