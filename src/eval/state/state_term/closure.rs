use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::{locations_clone::LocationsClone, state_term::{StateTerm, StateTermStore}};

#[derive(Clone, Debug)]
pub struct Closure {
    pub term: Term,
    pub vars: HashMap<String, StateTerm>
}

impl Closure {
    pub fn from_term(term: Term) -> Self {
        Closure {
            term, vars: HashMap::new()
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

    fn expand_value(&self, term: &Term) -> StateTerm {
        match term {
            Term::Var(var) => self.lookup(&var).unwrap(),
            Term::Succ(term) => match self.expand_value(&term) {
                StateTerm::Term(term) => StateTerm::from_term(Term::Succ(Box::new(term.term()))),
                StateTerm::Closure(_) => unreachable!()
            },
            Term::TypedVar(shape) => match shape.borrow().as_ref() {
                Some(term) => match term {
                    Term::Zero => StateTerm::from_term(Term::Zero),
                    Term::Succ(term) => match self.expand_value(&term) {
                        StateTerm::Term(term_ptr) => StateTerm::from_term(Term::Succ(Box::new(term_ptr.term()))),
                        StateTerm::Closure(_) => unreachable!()
                    },
                    _ => unreachable!()
                },
                None => StateTerm::from_term(Term::TypedVar(Rc::clone(shape)))
            },
            Term::Thunk(_) => StateTerm::Closure(Closure { term: term.clone(), vars: self.vars.clone() }),
            _ => StateTerm::from_term(term.clone())
        }
    }
}

impl LocationsClone for Closure {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
        let vars = self.vars.iter()
            .fold(HashMap::new(), |mut acc, (var, val)| {
                acc.insert(var.clone(), val.clone_with_locations(new_locations));
                acc
            });

        Closure {
            term: self.term.clone_with_locations(new_locations),
            vars
        }
    }
}
