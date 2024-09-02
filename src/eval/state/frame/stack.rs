use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::eval::state::state_term::StateTerm;

use super::env::env_value::TypeVal;

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

    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>>) -> Self {
        let stack = self.stack.iter()
            .fold(vec![], |mut acc , term| {
                acc.push(term.clone_with_locations(new_locations));
                acc
            });

        Stack { stack }
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

impl StackTerm {
    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>>) -> Self     {
        match self {
            StackTerm::Cont(var, term) => StackTerm::Cont(var.clone(), term.clone_with_locations(new_locations)),
            StackTerm::Term(term) => StackTerm::Term(term.clone_with_locations(new_locations)),
            StackTerm::Release(var) => StackTerm::Release(var.clone())
        }
    }
}