use std::collections::HashMap;

use crate::{cbpv::Term, parser::syntax::r#type::Type};

use super::{env::env_value::EnvValue, env_lookup::EnvLookup, state_term::StateTerm};

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

    pub fn store(&mut self, var: String, val: StateTerm) {
        self.vars.insert(var, EnvValue::Term(val));
    }

    pub fn bind(&mut self, var: String, r#type: Type) {
        todo!()
    }
}

impl EnvLookup for ClosureVars {
    fn lookup(&self, var: &String) -> Option<EnvValue> {
        match self.vars.get(var) {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}