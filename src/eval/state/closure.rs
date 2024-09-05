use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::state_term::StateTerm;

#[derive(Clone, Debug)]
pub struct Closure {
    pub term: Term,
    pub vars: HashMap<String, StateTerm>
}

impl Closure {
    pub fn new(term: Term) -> Self {
        Closure {
            term, vars: HashMap::new()
        }
    }

    pub fn store(&mut self, var: String, val: StateTerm) {
        self.vars.insert(var, val);
    }

    pub fn lookup(&self, var: &String) -> Option<StateTerm> {
        match self.vars.get(var) {
            Some(val) => Some(val.clone()),
            None => None
        }
    }

    pub fn expand_value(self, term: Term) -> StateTerm {
        match term {
            Term::Var(var) => self.lookup(&var).unwrap(),
            Term::Succ(term) => match self.expand_value(*term) {
                StateTerm::Term(term) => StateTerm::Term(Term::Succ(Box::new(term))),
                StateTerm::Closure(_) => unreachable!()
            },
            Term::TypedVar(shape) => if shape.borrow().is_some() {
                match shape.borrow().clone().unwrap() {
                    Term::Zero => StateTerm::Term(Term::Zero),
                    Term::Succ(term) => match self.expand_value(*term) {
                        StateTerm::Term(term) => StateTerm::Term(Term::Succ(Box::new(term))),
                        StateTerm::Closure(_) => unreachable!()
                    },
                    _ => unreachable!()
                }
            } else {
                StateTerm::Term(Term::TypedVar(shape))
            },
            Term::Thunk(_) => StateTerm::Closure(Closure { term, vars: self.vars }),
            _ => StateTerm::Term(term)
        }
    }

    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
        let vars = self.vars.iter()
            .fold(HashMap::new(), |mut acc, (var, val)| {
                acc.insert(var.clone(), val.clone_with_locations(new_locations));
                acc
            });

        Closure {
            term: self.term.clone(),
            vars
        }
    }
}
