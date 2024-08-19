use std::collections::HashMap;

use crate::{cbpv::Term, parser::syntax::r#type::Type};

use super::State;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Env {
    env: HashMap<String, EnvValue>,
    prev: Option<Box<Env>>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnvValue {
    Term(Term),
    Type(Type),
    Closure(Term, Env)
}

impl Env {
    pub fn new() -> Self {
        Env { env: HashMap::new(), prev: None }
    }

    pub fn push(old: Env) -> Self {
        Env { env: HashMap::new(), prev: Some(Box::new(old)) }
    }

    pub fn pop(self) -> Option<Box<Env>> {
        self.prev
    }

    pub fn in_scope(&self, var: &String) -> bool {
        self.env.contains_key(var)
    }

    pub fn store(&mut self, var: String, val: Term) {
        self.env.insert(var, EnvValue::Term(val));
    }

    pub fn bind(&mut self, var: String, r#type: Type) {
        self.env.insert(var, EnvValue::Type(r#type));
    }

    pub fn store_closure(&mut self, var: String, term: Term, env: Env) {
        self.env.insert(var, EnvValue::Closure(term, env));
    }

    pub fn lookup(&self, var: &String) -> Option<EnvValue> {
        match self.env.get(var) {
            Some(term) => Some(term.clone()),
            None => match &self.prev {
                Some(prev) => prev.lookup(var),
                None => None
            }
        }
    }
}