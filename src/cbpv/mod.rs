use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

pub mod translate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Var(String),
    TypedVar(Rc<RefCell<Option<Term>>>),
    Zero,
    Succ(Box<Term>),
    Nil,
    Cons(Box<Term>, Box<Term>),
    Bool(bool),
    Add(String, String),
    Eq(String, String),
    NEq(String, String),
    Not(String),
    If {
        cond: String,
        then: Box<Term>,
        r#else: Box<Term>
    },
    Bind {
        var: String,
        val: Box<Term>,
        body: Box<Term>,
    },
    Exists {
        var: String,
        body: Box<Term>
    },
    Equate {
        lhs: String,
        rhs: String,
        body: Box<Term>
    },
    Lambda {
        var: String,
        free_vars: HashSet<String>,
        body: Box<Term>
    },
    PM(PM),
    Choice(Vec<Term>),
    Thunk(Box<Term>),
    Return(Box<Term>),
    Force(String),
    App(Box<Term>, String),
    Fail
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PM {
    PMNat(PMNat),
    PMList(PMList)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMNat {
    pub var: String,
    pub zero: Box<Term>,
    pub succ: PMNatSucc
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMNatSucc {
    pub var: String,
    pub body: Box<Term>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMList {
    pub var: String,
    pub nil: Box<Term>,
    pub cons: PMListCons
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMListCons {
    pub var: String,
    pub body: Box<Term>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMSucc {
    pub var: String,
    pub body: Box<Term>
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
            Term::Lambda { var: _, free_vars, body: _ } => free_vars.clone(),
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

    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
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
            Term::Succ(term) => Term::Succ(Box::new(term.clone_with_locations(new_locations))),
            _ => self.clone()
        }
    }
}
