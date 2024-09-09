use crate::cbpv::Term;

use super::{closure::Closure, state_term::StateTerm, term_ptr::TermPtr};

pub enum Value {
    Term(Term),
    Closure(Closure)
}

impl Value {
    pub fn to_state_term(self) -> StateTerm {
        match self {
            Value::Term(term) => StateTerm::Term(TermPtr::new(term)),
            Value::Closure(closure) => StateTerm::Closure(closure)
        }
    }
}

pub trait ValueStore {
    fn store(&mut self, var: String, val: Value);

    fn lookup(&self, var: &String) -> Option<Value>;

    fn expand_value(&self, term: Term) -> Value;
}
