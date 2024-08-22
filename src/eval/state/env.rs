use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{cbpv::Term, parser::syntax::r#type::Type};

use super::state_term::StateTerm;

#[derive(Debug)]
pub struct Env {
    env: HashMap<String, EnvValue>,
    prev: Option<Rc<RefCell<Env>>>
}

#[derive(Clone, Debug)]
pub enum EnvValue {
    Term(StateTerm),
    Type(Type),
}

impl Env {
    pub fn new() -> Self {
        Env { env: HashMap::new(), prev: None }
    }

    pub fn push(old: &Rc<RefCell<Env>>) -> Self {
        Env { env: HashMap::new(), prev: Some(Rc::clone(old)) }
    }

    pub fn pop(&self) -> Option<Rc<RefCell<Env>>> {
        match &self.prev {
            Some(prev) => Some(Rc::clone(prev)),
            None => None
        }
    }

    pub fn in_scope(&self, var: &String) -> bool {
        self.env.contains_key(var)
    }

    pub fn store(&mut self, var: String, val: StateTerm) {
        let val = match val {
            StateTerm::Closure(closure) => StateTerm::Closure(closure.fix_cycle(self)),
            _ => val
        };
        self.env.insert(var, EnvValue::Term(val));
    }

    pub fn bind(&mut self, var: String, r#type: Type) {
        self.env.insert(var, EnvValue::Type(r#type));
    }

    pub fn lookup(&self, var: &String) -> Option<EnvValue> {
        match self.env.get(var) {
            Some(term) => match term {
                EnvValue::Term(term) => match term {
                    StateTerm::Term(term) => match term {
                        Term::Var(var) => self.lookup(var),
                        _ => Some(EnvValue::Term(StateTerm::Term(term.clone())))
                    },
                    StateTerm::Closure(_) => Some(EnvValue::Term(term.clone()))
                },
                EnvValue::Type(_) => Some(term.clone())
            },
            None => match &self.prev {
                Some(prev) => prev.borrow().lookup(var),
                None => None
            }
        }
    }
}