use std::collections::HashSet;

use crate::parser::syntax::r#type::Type;

pub mod translate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Var(String),
    Zero,
    Succ(Box<Term>),
    Bool(bool),
    Add(Box<Term>, Box<Term>),
    Eq(Box<Term>, Box<Term>),
    NEq(Box<Term>, Box<Term>),
    Not(Box<Term>),
    If {
        cond: Box<Term>,
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
        r#type: Type,
        body: Box<Term>
    },
    Equate {
        lhs: Box<Term>,
        rhs: Box<Term>,
        body: Box<Term>
    },
    Lambda {
        var: String,
        free_vars: HashSet<String>,
        body: Box<Term>
    },
    PM {
        var: String,
        zero: Box<Term>,
        succ: PMSucc
    },
    Choice(Vec<Term>),
    Thunk(Box<Term>),
    Return(Box<Term>),
    Force(Box<Term>),
    App(Box<Term>, Box<Term>),
    Fail
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
            Term::If { cond: _, then, r#else } => {
                let mut free_vars = then.free_vars();
                free_vars.extend(r#else.free_vars());
                free_vars
            },
            Term::Bind { var, val, body } => {
                let mut free_vars = val.free_vars();
                free_vars.extend(body.free_vars());
                free_vars.remove(var);
                free_vars
            },
            Term::Exists { var, r#type: _, body } => {
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
            Term::Force(term) => term.free_vars(),
            Term::App(lhs, rhs) => {
                let mut free_vars = lhs.free_vars();
                free_vars.extend(rhs.free_vars());
                free_vars
            },
            _ => HashSet::new(),
        }
    }
}