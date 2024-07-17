use crate::parser::syntax::r#type::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term<'a> {
    Var(String),
    Succ(usize, Option<Box<Term<'a>>>),
    Bool(bool),
    Add(Box<Term<'a>>, Box<Term<'a>>),
    AddValue(Box<Term<'a>>, Box<Term<'a>>),
    Eq(Box<Term<'a>>, Box<Term<'a>>),
    NEq(Box<Term<'a>>, Box<Term<'a>>),
    Not(Box<Term<'a>>),
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

pub fn substitute<'a>(term: Term<'a>, var: &str, sub: &Term<'a>) -> Term<'a> {
    match term {
        Term::Var(s) => if s == var { sub.clone() } else { Term::Var(s) },
        Term::Succ(n, t) => match t {
            Some(t) => Term::Succ(n, Some(Box::new(substitute(*t, var, sub)))),
            None => Term::Succ(n, None)
        },
        Term::Bool(_) => term,
        Term::Eq(lhs, rhs) => Term::Eq(
            Box::new(substitute(*lhs, var, sub)),
            Box::new(substitute(*rhs, var, sub))
        ),
        Term::NEq(lhs, rhs) => Term::NEq(
            Box::new(substitute(*lhs, var, sub)),
            Box::new(substitute(*rhs, var, sub))
        ),
        Term::Not(t) => Term::Not(Box::new(substitute(*t, var, sub))),
        Term::If { cond, then, r#else } => Term::If {
            cond: Box::new(substitute(*cond, var, sub)),
            then: Box::new(substitute(*then, var, sub)),
            r#else: Box::new(substitute(*r#else, var, sub))
        },
        Term::Bind { var: v, val, body } => {
            let flag = v == var;

            Term::Bind {
                var: v,
                val: Box::new(substitute(*val, var, sub)),
                body: if flag { body } else {
                    Box::new(substitute(*body, var, sub))
                }
            }
        },
        Term::Exists { var: v, r#type, body } => {
            Term::Exists {
                var: v,
                r#type,
                body: if v == var { body } else {
                    Box::new(substitute(*body, var, sub))
                }
            }
        },
        Term::Equate { lhs, rhs, body } => {
            Term::Equate {
                lhs: Box::new(substitute(*lhs, var, sub)),
                rhs: Box::new(substitute(*rhs, var, sub)),
                body: Box::new(substitute(*body, var, sub))
            }
        },
        Term::Lambda { args, body } => {
            let flag = args.contains(&var);

            Term::Lambda {
                args,
                body: if flag { 
                    body
                } else {
                    Box::new(substitute(*body, var, sub))
                }
            }
        },
        Term::Choice(c) => Term::Choice(
            c.into_iter()
                .map(|t| substitute(t, var, sub))
                .collect()
        ),
        Term::Thunk(t) => Term::Thunk(
            Box::new(substitute(*t, var, sub))
        ),
        Term::Return(t) => Term::Return(
            Box::new(substitute(*t, var, sub))
        ),
        Term::Force(t) => Term::Force(
            Box::new(substitute(*t, var, sub))
        ),
        Term::Add(lhs, rhs) => Term::Add(
            Box::new(substitute(*lhs, var, sub)),
            Box::new(substitute(*rhs, var, sub))
        ),
        Term::AddValue(lhs, rhs) => Term::AddValue(
            Box::new(substitute(*lhs, var, sub)),
            Box::new(substitute(*rhs, var, sub))
        ),
        Term::App(lhs, rhs) => Term::App(
            Box::new(substitute(*lhs, var, sub)),
            Box::new(substitute(*rhs, var, sub))
        ),
        Term::Fail => Term::Fail
    }
}