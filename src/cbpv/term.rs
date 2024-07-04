use crate::parser::syntax::r#type::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term<'a> {
    Var(String),
    Nat(i64),
    If {
        cond: Box<Term<'a>>,
        then: Box<Term<'a>>,
        r#else: Box<Term<'a>>
    },
    Bind {
        var: String,
        val: Box<Term<'a>>,
        body: Box<Term<'a>>,
    },
    Exists {
        var: &'a str,
        r#type: Type<'a>,
        body: Box<Term<'a>>
    },
    Equate {
        lhs: Box<Term<'a>>,
        rhs: Box<Term<'a>>,
        body: Box<Term<'a>>
    },
    Lambda {
        args: Vec<&'a str>,
        body: Box<Term<'a>>
    },
    Choice(Vec<Term<'a>>),
    Thunk(Box<Term<'a>>),
    Return(Box<Term<'a>>),
    Force(Box<Term<'a>>),
    App(Box<Term<'a>>, Box<Term<'a>>),
    Fail
}