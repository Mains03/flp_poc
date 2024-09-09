use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::{locations_clone::LocationsClone, state_term::StateTerm, value::{Value, ValueStore}};

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
}

impl ValueStore for Closure {
    fn store(&mut self, var: String, val: Value) {
        self.vars.insert(var, val.to_state_term());
    }

    fn lookup(&self, var: &String) -> Option<Value> {
        match self.vars.get(var) {
            Some(state_term) => Some(state_term.as_value()),
            None => None
        }
    }

    fn expand_value(&self, term: Term) -> Value {
        match term {
            Term::Var(var) => self.lookup(&var).unwrap(),
            Term::Succ(term) => match self.expand_value(*term) {
                Value::Term(term) => Value::Term(Term::Succ(Box::new(term))),
                Value::Closure(_) => unreachable!()
            },
            Term::TypedVar(shape) => if shape.borrow().is_some() {
                match shape.borrow().clone().unwrap() {
                    Term::Zero => Value::Term(Term::Zero),
                    Term::Succ(term) => match self.expand_value(*term) {
                        Value::Term(term) => Value::Term(Term::Succ(Box::new(term))),
                        Value::Closure(_) => unreachable!()
                    },
                    _ => unreachable!()
                }
            } else {
                Value::Term(Term::TypedVar(shape))
            },
            Term::Thunk(_) => Value::Closure(Closure { term, vars: self.vars.clone() }),
            _ => Value::Term(term)
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
            term: self.term.clone(),
            vars
        }
    }
}
