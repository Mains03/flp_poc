use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::{term_ptr::TermPtr, Term};

use super::{closure::Closure, locations_clone::LocationsClone};

#[derive(Clone, Debug)]
pub enum StateTerm {
    Term(TermPtr),
    Closure(Closure)
}

pub trait StateTermStore {
    fn store(&mut self, var: String, val: StateTerm);

    fn lookup(&self, var: &String) -> StateTerm;
}

impl StateTerm {
    pub fn from_term(term: Term) -> Self {
        StateTerm::Term(TermPtr::from_term(term))
    }

    pub fn from_term_ptr(term_ptr: TermPtr) -> Self {
        StateTerm::Term(term_ptr)
    }
}

impl LocationsClone for StateTerm {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>) -> Self {
        match self {
            StateTerm::Term(term) => StateTerm::Term(term.clone_with_locations(new_locations)),
            StateTerm::Closure(closure) => StateTerm::Closure(closure.clone_with_locations(new_locations))
        }
    }
}