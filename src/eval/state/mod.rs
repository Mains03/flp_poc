use std::collections::HashMap;

use env::Env;
use stack::Stack;
use state_term::state_term::{StateTerm, StateTermStore};
use step::step;

pub use state_term::locations_clone::LocationsClone;

use crate::cbpv::{term_ptr::TermPtr, Term};

mod env;
mod equate;
mod stack;
mod state_term;
mod step;

#[derive(Debug)]
pub struct State {
    env: Env, 
    term: StateTerm,
    stack: Stack
}

impl State {
    pub fn new(mut cbpv: HashMap<String, Term>) -> Self {
        let term = cbpv.remove("main").unwrap();

        let env = cbpv.into_iter()
            .fold(Env::new(), |mut env, (var, val)| {
                env.store(var, StateTerm::from_term(val));
                env
            });

        State {
            env,
            term: StateTerm::from_term(term),
            stack: Stack::new()
        }
    }

    pub fn step(self) -> Vec<State> {
        let term = match &self.term {
            StateTerm::Term(term) => term.clone(),
            StateTerm::Closure(closure) => closure.term_ptr.clone()
        };

        let in_closure = match self.term {
            StateTerm::Term(_) => false,
            StateTerm::Closure(_) => true
        };

        let closure_env = match self.term {
            StateTerm::Term(_) => None,
            StateTerm::Closure(closure) => Some(closure.env)
        };

        step(term, self.env, self.stack, in_closure, closure_env)
    }

    pub fn is_fail(&self) -> bool {
        match &self.term {
            StateTerm::Term(term_ptr) => match term_ptr.term() {
                Term::Fail => true,
                _ => false
            },
            StateTerm::Closure(_) => false
        }
    }

    pub fn is_value(&self) -> bool {
        if self.stack.is_empty() {
            match &self.term {
                StateTerm::Term(term_ptr) => match term_ptr.term() {
                    Term::Return(term_ptr) => match term_ptr.term() {
                        Term::Var(_) => false,
                        _ => true
                    },
                    _ => false
                },
                _ => false
            }
        } else {
            false
        }
    }

    pub fn term(self) -> TermPtr {
        match self.term {
            StateTerm::Term(term_ptr) => term_ptr.clone(),
            StateTerm::Closure(_) => unreachable!()
        }
    }
}
