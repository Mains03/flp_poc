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
        body: Box<Term>
    },
    Choice(Vec<Term>),
    Thunk(Box<Term>),
    Return(Box<Term>),
    Force(Box<Term>),
    App(Box<Term>, Box<Term>),
    Fail
}