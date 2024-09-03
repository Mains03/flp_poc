use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::closure::Closure;

#[derive(Clone, Debug)]
pub enum StateTerm {
    Term(Term),
    Closure(Closure)
}

impl StateTerm {
    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
        match self {
            StateTerm::Term(term) => StateTerm::Term(term.clone_with_locations(new_locations)),
            StateTerm::Closure(closure) => StateTerm::Closure(closure.clone_with_locations(new_locations))
        }
    }
}