use std::{cell::RefCell, rc::Rc};

use crate::cbpv::Term;

use super::env::Env;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closure {
    pub term: Term,
    pub env: Rc<RefCell<Env>>
}

impl Closure {
    pub fn new(term: Term, env: &Rc<RefCell<Env>>) -> Self {
        Closure { term, env: Rc::clone(env) }
    }
}