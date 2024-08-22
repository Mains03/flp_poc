use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::cbpv::Term;

use super::env::Env;

#[derive(Clone, Debug)]
pub struct Closure {
    pub term: Term,
    env: EnvPointer
}

#[derive(Clone, Debug)]
enum EnvPointer {
    Strong(Rc<RefCell<Env>>),
    Weak(Weak<RefCell<Env>>)
}

impl Closure {
    pub fn new(term: Term, env: &Rc<RefCell<Env>>) -> Self {
        Closure { term, env: EnvPointer::Strong(Rc::clone(env)) }
    }

    pub fn env(&self) -> Rc<RefCell<Env>> {
        match &self.env {
            EnvPointer::Strong(env) => Rc::clone(env),
            EnvPointer::Weak(env) => env.upgrade().unwrap()
        }
    }

    pub fn fix_cycle(self, target: *mut Env) -> Self {
        match &self.env {
            EnvPointer::Strong(env) => if env.as_ptr() == target {
                Closure {
                    term: self.term,
                    env: EnvPointer::Weak(Rc::downgrade(env))
                }
            } else {
                self
            },
            EnvPointer::Weak(_) => self
        }
    }
}