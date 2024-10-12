use std::{cell::RefCell, collections::HashMap, rc::Rc};

use pm::PM;
use term_ptr::TermPtr;

use crate::{eval::{Env, LocationsClone}, parser::syntax::arg::Arg};

pub mod pm;
pub mod term_ptr;
pub mod translate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Var(String),
    TypedVar(Rc<RefCell<Option<TermPtr>>>),
    Zero,
    Succ(TermPtr),
    Nil,
    Cons(TermPtr, TermPtr),
    Bool(bool),
    Pair(TermPtr, TermPtr),
    Add(String, String),
    Eq(String, String),
    NEq(String, String),
    And(TermPtr, TermPtr),
    Or(TermPtr, TermPtr),
    Not(String),
    If {
        cond: String,
        then: TermPtr,
        r#else: TermPtr
    },
    Bind {
        var: String,
        val: TermPtr,
        body: TermPtr,
    },
    Exists {
        var: String,
        body: TermPtr
    },
    Equate {
        lhs: String,
        rhs: String,
        body: TermPtr
    },
    Lambda {
        arg: Arg,
        body: TermPtr
    },
    PM(PM),
    Choice(Vec<TermPtr>),
    Thunk(TermPtr),
    Return(TermPtr),
    Force(String),
    App(TermPtr, String),
    Fail
}

impl Term {
    pub fn contains_typed_var(&self) -> bool {
        match self {
            Term::TypedVar(val) => match val.borrow().as_ref() {
                Some(term) => term.contains_typed_var(),
                None => true
            },
            Term::Pair(lhs, rhs) => lhs.contains_typed_var() || rhs.contains_typed_var(),
            Term::Succ(term) => term.contains_typed_var(),
            Term::Cons(x, xs) => x.contains_typed_var() || xs.contains_typed_var(),
            _ => false
        }
    }
}

impl LocationsClone for Term {
    fn clone_with_locations(
        &self,
        new_val_locs: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>,
        new_env_locs: &mut HashMap<*mut Env, Rc<RefCell<Env>>>
    ) -> Self {
        match self {
            Term::TypedVar(location) => match new_val_locs.get(&location.as_ptr()) {
                Some(new_location) => Term::TypedVar(Rc::clone(new_location)),
                None => match location.borrow().clone() {
                    Some(shape) => {
                        let new_location = Rc::new(RefCell::new(
                            Some(shape.clone_with_locations(new_val_locs, new_env_locs))
                        ));

                        new_val_locs.insert(location.as_ptr(), Rc::clone(&new_location));

                        Term::TypedVar(new_location)
                    },
                    None => {
                        let new_location = Rc::new(RefCell::new(None));

                        new_val_locs.insert(location.as_ptr(), Rc::clone(&new_location));

                        Term::TypedVar(new_location)
                    }
                }
            },
            Term::Pair(lhs, rhs) => Term::Pair(
                lhs.clone_with_locations(new_val_locs, new_env_locs),
                rhs.clone_with_locations(new_val_locs, new_env_locs)
            ),
            Term::Succ(term) => Term::Succ(term.clone_with_locations(new_val_locs, new_env_locs)),
            Term::Cons(x, xs) => Term::Cons(
                x.clone_with_locations(new_val_locs, new_env_locs),
                xs.clone_with_locations(new_val_locs, new_env_locs)
            ),
            _ => self.clone()
        }
    }
}
