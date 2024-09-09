use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::{closure::Closure, locations_clone::LocationsClone, term_ptr::TermPtr, value::Value};

#[derive(Clone, Debug)]
pub enum StateTerm {
    Term(TermPtr),
    Closure(Closure)
}

impl StateTerm {
    pub fn from_term(term: Term) -> Self {
        StateTerm::Term(TermPtr::new(term))
    }

    pub fn as_value(&self) -> Value {
        match self {
            StateTerm::Term(term_ptr) => Value::Term(term_ptr.term()),
            StateTerm::Closure(closure) => Value::Closure(closure.clone())
        }
    }
}

impl LocationsClone for StateTerm {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
        match self {
            StateTerm::Term(term) => StateTerm::Term(term.clone_with_locations(new_locations)),
            StateTerm::Closure(closure) => StateTerm::Closure(closure.clone_with_locations(new_locations))
        }
    }
}