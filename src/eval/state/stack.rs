use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::term_ptr::TermPtr;

use super::{env::Env, state_term::{locations_clone::LocationsClone, state_term::StateTerm}};

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
}

impl LocationsClone for Stack {
    fn clone_with_locations(
        &self,
        new_val_locs: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>,
        new_env_locs: &mut HashMap<*mut Env, Rc<RefCell<Env>>>
    ) -> Self {
        let stack = self.stack.iter()
            .fold(vec![], |mut acc , term| {
                acc.push(term.clone_with_locations(new_val_locs, new_env_locs));
                acc
            });

        Stack { stack }
    }
}

impl LocationsClone for StackTerm {
    fn clone_with_locations(
        &self,
        new_val_locs: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>,
        new_env_locs: &mut HashMap<*mut Env, Rc<RefCell<Env>>>
    ) -> Self     {
        match self {
            StackTerm::Cont(var, term) => StackTerm::Cont(var.clone(), term.clone_with_locations(new_val_locs, new_env_locs)),
            StackTerm::Term(term) => StackTerm::Term(term.clone_with_locations(new_val_locs, new_env_locs)),
            StackTerm::Release(var) => StackTerm::Release(var.clone())
        }
    }
}