use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::state_term::{locations_clone::LocationsClone, state_term::StateTerm};

#[derive(Debug)]
pub struct Stack {
    stack: Vec<StackTerm>
}

#[derive(Debug)]
pub enum StackTerm {
    Cont(String, StateTerm),
    Term(StateTerm),
    Release(String),
}

impl Stack {
    pub fn new() -> Self {
        Stack { stack: vec![] }
    }

    pub fn push(&mut self, term: StackTerm) {
        self.stack.push(term);
    }

    pub fn pop(&mut self) -> Option<StackTerm> {
        if self.stack.len() == 0 {
            None
        } else {
            Some(self.stack.remove(self.stack.len()-1))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.len() == 0
    }

    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
        let stack = self.stack.iter()
            .fold(vec![], |mut acc , term| {
                acc.push(term.clone_with_locations(new_locations));
                acc
            });

        Stack { stack }
    }
}

impl LocationsClone for StackTerm {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self     {
        match self {
            StackTerm::Cont(var, term) => StackTerm::Cont(var.clone(), term.clone_with_locations(new_locations)),
            StackTerm::Term(term) => StackTerm::Term(term.clone_with_locations(new_locations)),
            StackTerm::Release(var) => StackTerm::Release(var.clone())
        }
    }
}