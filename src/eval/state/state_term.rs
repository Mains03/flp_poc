use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::{closure::Closure, frame::env::env_value::TypeVal};

#[derive(Clone, Debug)]
pub enum StateTerm {
    Term(Term),
    Closure(Closure)
}

impl StateTerm {
    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>>) -> Self {
        match self {
            StateTerm::Term(term) => StateTerm::Term(term.clone()),
            StateTerm::Closure(closure) => StateTerm::Closure(closure.clone_with_locations(new_locations))
        }
    }
}