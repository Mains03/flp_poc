use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::{closure::Closure, locations_clone::LocationsClone, term_ptr::TermPtr};

#[derive(Clone, Debug)]
pub enum StateTerm {
    Term(TermPtr),
    Closure(Closure)
}

pub trait StateTermStore {
    fn store(&mut self, var: String, val: StateTerm);

    fn lookup(&self, var: &String) -> Option<StateTerm>;

    fn expand_value(&self, term: &Term) -> StateTerm;
}

impl StateTerm {
    pub fn from_term(term: Term) -> Self {
        StateTerm::Term(TermPtr::new(term))
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