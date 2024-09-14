use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use pm::PM;
use term_ptr::TermPtr;

use crate::{eval::LocationsClone, parser::syntax::arg::Arg};

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
    Fold,
    Eq(String, String),
    NEq(String, String),
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
        free_vars: HashSet<String>,
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
    pub fn free_vars(&self) -> HashSet<String> {
        match self {
            Term::Var(var) => HashSet::from_iter(vec![var.to_string()]),
            Term::Add(lhs, rhs) => HashSet::from_iter(vec![lhs.clone(), rhs.clone()]),
            Term::Eq(lhs, rhs) => HashSet::from_iter(vec![lhs.clone(), rhs.clone()]),
            Term::NEq(lhs, rhs) => HashSet::from_iter(vec![lhs.clone(), rhs.clone()]),
            Term::Not(term) => HashSet::from_iter(vec![term.clone()]),
            Term::If { cond, then, r#else } => {
                let mut free_vars = HashSet::from_iter(vec![cond.clone()]);
                free_vars.extend(then.free_vars());
                free_vars.extend(r#else.free_vars());
                free_vars
            },
            Term::Bind { var, val, body } => {
                let mut free_vars = val.free_vars();
                free_vars.extend(body.free_vars());
                free_vars.remove(var);
                free_vars
            },
            Term::Exists { var, body } => {
                let mut free_vars = body.free_vars();
                free_vars.remove(var);
                free_vars
            },
            Term::Equate { lhs: _, rhs: _, body } => body.free_vars(),
            Term::Lambda { arg: _, free_vars, body: _ } => free_vars.clone(),
            Term::Choice(v) => v.iter()
                .fold(HashSet::new(), |mut acc, x| {
                    acc.extend(x.free_vars());
                    acc
                }),
            Term::Thunk(term) => term.free_vars(),
            Term::Return(term) => term.free_vars(),
            Term::Force(term) => HashSet::from_iter(vec![term.clone()]),
            Term::App(lhs, rhs) => {
                let mut free_vars = lhs.free_vars();
                free_vars.insert(rhs.clone());
                free_vars
            },
            _ => HashSet::new(),
        }
    }

    pub fn contains_typed_var(&self) -> bool {
        match self {
            Term::TypedVar(val) => match val.borrow().as_ref() {
                Some(term) => term.contains_typed_var(),
                None => true
            },
            Term::Succ(term) => term.contains_typed_var(),
            Term::Cons(x, xs) => x.contains_typed_var() || xs.contains_typed_var(),
            _ => false
        }
    }
}

impl LocationsClone for Term {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>) -> Self {
        match self {
            Term::TypedVar(location) => match new_locations.get(&location.as_ptr()) {
                Some(new_location) => Term::TypedVar(Rc::clone(new_location)),
                None => match location.borrow().clone() {
                    Some(shape) => {
                        let new_location = Rc::new(RefCell::new(
                            Some(shape.clone_with_locations(new_locations))
                        ));

                        new_locations.insert(location.as_ptr(), Rc::clone(&new_location));

                        Term::TypedVar(new_location)
                    },
                    None => {
                        let new_location = Rc::new(RefCell::new(None));

                        new_locations.insert(location.as_ptr(), Rc::clone(&new_location));

                        Term::TypedVar(new_location)
                    }
                }
            },
            Term::Succ(term) => Term::Succ(term.clone_with_locations(new_locations)),
            Term::Cons(x, xs) => Term::Cons(
                x.clone_with_locations(new_locations),
                xs.clone_with_locations(new_locations)
            ),
            _ => self.clone()
        }
    }
}
