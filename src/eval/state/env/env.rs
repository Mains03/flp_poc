use std::collections::HashMap;

use crate::{eval::state::state_term::StateTerm, parser::syntax::r#type::Type};

use super::env_value::EnvValue;

#[derive(Debug)]
pub struct Env {
    env: HashMap<String, EnvValue>,
    prev: Option<Box<Env>>
}

impl Env {
    pub fn new() -> Self {
        Env { env: HashMap::new(), prev: None }
    }

    pub fn store(self, var: String, val: StateTerm) -> Self {
        let val = EnvValue::Term(val);

        if self.env.contains_key(&var) {
            let prev = Some(Box::new(Env { env: self.env, prev: self.prev }));

            let mut env = HashMap::new();
            env.insert(var, val);

            Env { env, prev }
        } else {
            let mut env = self.env;
            env.insert(var, val);

            Env { env, prev: self.prev }
        }
    }

    pub fn bind(&mut self, _: String, _: Type) {
        todo!()
    }

    pub fn lookup(&self, var: &String) -> Option<EnvValue> {
        match self.env.get(var) {
            Some(val) => Some(val.clone()),
            None => match &self.prev {
                Some(prev) => prev.lookup(var),
                None => None
            }
        }
    }

    pub fn release(mut self, var: &String) -> Self {
        if self.env.contains_key(var) {
            self.env.remove(var);
            if self.env.is_empty() {
                *self.prev.unwrap()
            } else {
                self
            }
        } else {
            let prev = self.prev.unwrap().release(var);
            
            Env { env: self.env, prev: Some(Box::new(prev)) }
        }
    }
}