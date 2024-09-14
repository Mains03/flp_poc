use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{cbpv::{term_ptr::TermPtr, Term}, parser::syntax::arg::Arg};

use super::{locations_clone::LocationsClone, state_term::{StateTerm, StateTermStore}};

#[derive(Clone, Debug)]
pub struct Closure {
    pub term_ptr: TermPtr,
    pub vars: HashMap<String, StateTerm>
}

impl Closure {
    pub fn from_term_ptr(term_ptr: TermPtr) -> Self {
        Closure {
            term_ptr, vars: HashMap::new()
        }
    }

    pub fn store_arg(&mut self, arg: Arg, val: StateTerm) {
        match arg {
            Arg::Ident(var) => self.store(var, val),
            Arg::Pair(lhs, rhs) => match val {
                StateTerm::Term(term) => match term.term() {
                    Term::Pair(lhs_val, rhs_val) => {
                        self.store_arg(*lhs, StateTerm::from_term_ptr(lhs_val.clone()));
                        self.store_arg(*rhs, StateTerm::from_term_ptr(rhs_val.clone()));
                    },
                    _ => unreachable!()
                },
                StateTerm::Closure(_) => unreachable!()
            }
        }
    }
}

impl StateTermStore for Closure {
    fn store(&mut self, var: String, val: StateTerm) {
        self.vars.insert(var, val);
    }

    fn lookup(&self, var: &String) -> Option<StateTerm> {
        match self.vars.get(var) {
            Some(state_term) => Some(state_term.clone()),
            None => None
        }
    }

    fn expand_value(&self, term_ptr: TermPtr) -> StateTerm {
        match term_ptr.term() {
            Term::Var(var) => self.lookup(&var).unwrap(),
            Term::Pair(lhs, rhs) => match self.expand_value(lhs.clone()) {
                StateTerm::Term(lhs) => match self.expand_value(rhs.clone()) {
                    StateTerm::Term(rhs) => StateTerm::from_term(Term::Pair(lhs, rhs)),
                    StateTerm::Closure(_) => unreachable!()
                },
                StateTerm::Closure(_) => unreachable!()
            },
            Term::Succ(term_ptr) => match self.expand_value(term_ptr.clone()) {
                StateTerm::Term(term_ptr) => StateTerm::from_term(Term::Succ(term_ptr)),
                StateTerm::Closure(_) => unreachable!()
            },
            Term::TypedVar(shape) => match shape.borrow().as_ref() {
                Some(term_ptr) => match term_ptr.term() {
                    Term::Zero => StateTerm::from_term(Term::Zero),
                    Term::Succ(term_ptr) => match self.expand_value(term_ptr.clone()) {
                        StateTerm::Term(term_ptr) => StateTerm::from_term(Term::Succ(term_ptr)),
                        StateTerm::Closure(_) => unreachable!()
                    },
                    _ => unreachable!()
                },
                None => StateTerm::from_term(Term::TypedVar(Rc::clone(shape)))
            },
            Term::Thunk(_) => StateTerm::Closure(Closure { term_ptr: term_ptr.clone(), vars: self.vars.clone() }),
            _ => StateTerm::from_term_ptr(term_ptr.clone())
        }
    }
}

impl LocationsClone for Closure {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>) -> Self {
        let vars = self.vars.iter()
            .fold(HashMap::new(), |mut acc, (var, val)| {
                acc.insert(var.clone(), val.clone_with_locations(new_locations));
                acc
            });

        Closure {
            term_ptr: self.term_ptr.clone_with_locations(new_locations),
            vars
        }
    }
}
