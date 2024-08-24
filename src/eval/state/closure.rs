use std::collections::HashMap;

use crate::cbpv::Term;

use super::env::env_value::EnvValue;

#[derive(Clone, Debug)]
pub struct Closure {
    pub term: Term,
    pub vars: HashMap<String, EnvValue>
}

impl Closure {
    pub fn lookup(&self, var: &String) -> Option<EnvValue> {
        match self.vars.get(var) {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}