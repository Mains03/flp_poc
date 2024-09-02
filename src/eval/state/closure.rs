use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::{env_lookup::EnvLookup, frame::env::env_value::{EnvValue, TypeVal}, state_term::StateTerm};

#[derive(Clone, Debug)]
pub struct Closure {
    pub term: Term,
    pub vars: ClosureVars
}

#[derive(Clone, Debug)]
pub struct ClosureVars {
    vars: HashMap<String, EnvValue>
}

impl Closure {
    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>>) -> Self {
        Closure {
            term: self.term.clone(),
            vars: self.vars.clone_with_locations(new_locations)
        }
    }
}

impl ClosureVars {
    pub fn new() -> Self {
        ClosureVars { vars: HashMap::new() }
    }

    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>>) -> Self {
        let vars = self.vars.iter()
            .fold(HashMap::new(), |mut acc, (var, val)| {
                acc.insert(var.clone(), val.clone_with_locations(new_locations));
                acc
            });

        ClosureVars { vars }
    }

    pub fn store(&mut self, var: String, val: StateTerm) {
        self.vars.insert(var, EnvValue::Term(val));
    }

    pub fn bind(&mut self, var: String, val: &Rc<RefCell<TypeVal>>) {
        self.vars.insert(var, EnvValue::Type(Rc::clone(val)));
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