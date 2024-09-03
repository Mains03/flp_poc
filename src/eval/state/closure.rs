use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::state_term::StateTerm;

#[derive(Clone, Debug)]
pub struct Closure {
    pub term: Term,
    pub vars: ClosureVars
}

#[derive(Clone, Debug)]
pub struct ClosureVars {
    vars: HashMap<String, StateTerm>
}

impl Closure {
    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
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

    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
        let vars = self.vars.iter()
            .fold(HashMap::new(), |mut acc, (var, val)| {
                acc.insert(var.clone(), val.clone_with_locations(new_locations));
                acc
            });

        ClosureVars { vars }
    }

    pub fn store(&mut self, var: String, val: StateTerm) {
        self.vars.insert(var, val);
    }
}
