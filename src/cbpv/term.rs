use std::io::stdin;

use crate::parser::syntax::r#type::Type;

use super::{equate::eval_equate, exists::eval_exists};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term<'a> {
    Var(String),
    Zero,
    Succ(Box<Term<'a>>),
    Bool(bool),
    Add(Box<Term<'a>>, Box<Term<'a>>),
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

impl<'a> Term<'a> {
    pub fn eval(self) -> Term<'a> {
        let mut term = self;
        loop {
            let new_term = term.clone().eval_step();

            if new_term == term {
                break
            } else {
                term = new_term;
            }
        }

        term
    }

    fn eval_step(self) -> Term<'a> {
        match self.propogate() {
            Term::Add(lhs, rhs) => match *lhs {
                Term::Zero => Term::Return(rhs),
                Term::Succ(t) => {
                    let var = "".to_string();

                    Term::Bind {
                        var: var.clone(),
                        val: Box::new(Term::Add(t, rhs)),
                        body: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Var(var))))))
                    }
                },
                _ => match *rhs {
                    Term::Zero => Term::Return(lhs),
                    Term::Succ(t) =>  {
                        let var = "".to_string();

                        Term::Bind {
                            var: var.clone(),
                            val: Box::new(Term::Add(lhs, t)),
                            body: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Var(var))))))
                        }
                    },
                    _ => unreachable!()
                }
            },
            Term::Eq(lhs, rhs) => Term::Return(Box::new(Term::Bool(lhs == rhs))),
            Term::NEq(lhs, rhs) => Term::Return(Box::new(Term::Bool(lhs != rhs))),
            Term::Not(t) => match *t {
                Term::Bool(b) => Term::Return(Box::new(Term::Bool(!b))),
                _ => unreachable!()
            },
            Term::If { cond, then, r#else } => match *cond {
                Term::Bool(b) => if b {
                    *then
                } else {
                    *r#else
                },
                _ => unreachable!()
            },
            Term::Bind { var, val, body } => match val.eval() {
                Term::Fail => Term::Fail,
                Term::Return(t) => body.substitute(&var, &t),
                Term::Choice(v) => Term::Choice(
                    v.into_iter()
                        .map(|t| Term::Bind { var: var.clone(), val: Box::new(t), body: body.clone() })
                        .collect()
                ),
                _ => unreachable!()
            },
            Term::Exists { var, r#type, body } => Term::Choice(vec![
                body.clone().substitute(var, &Term::Zero),
                Term::Exists { var, r#type, body: Box::new(body.substitute(var, &Term::Succ(Box::new(Term::Var(var.to_string()))))) }
            ]),
            Term::Equate { lhs, rhs, body } => if lhs == rhs {
                *body
            } else {
                Term::Fail
            },
            Term::Choice(mut v) => if v.len() == 0 {
                Term::Fail
            } else if v.len() == 1 {
                v.remove(0)
            } else {
                Term::Choice(v.into_iter()
                    .flat_map(|t|
                        t.eval_flat()
                            .into_iter()
                            .filter(|t| *t != Term::Fail))
                    .collect()
                )
            },
            Term::Force(t) => match *t {
                Term::Thunk(t) => *t,
                _ => unreachable!()
            },
            Term::App(lhs, rhs) => match lhs.eval() {
                Term::Lambda { mut args, body } => {
                    let var = args.remove(args.len() - 1);
                    let body = body.substitute(var, &rhs);

                    if args.len() == 0 {
                        body
                    } else {
                        Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                            args,
                            body: Box::new(body)
                        }))))
                    }
                },
                _ => unreachable!()
            },
            t => t
        }
    }

    fn eval_flat(self) -> Vec<Term<'a>> {
        match self {
            Term::Choice(v) => v.into_iter()
                .flat_map(|t| t.eval_flat())
                .collect(),
            _ => vec![self.eval()]
        }
    }

    fn propogate(self) -> Term<'a> {
        let mut term = self;
        loop {
            let new_term = term.clone().propogate_step();

            if new_term == term {
                break
            } else {
                term = new_term;
            }
        }

        term
    }
    
    fn propogate_step(self) -> Term<'a> {
        match self {
            Term::Add(lhs, rhs) => match *lhs {
                Term::Zero => Term::Return(rhs),
                Term::Succ(t) => Term::Bind {
                    var: "".to_string(),
                    val: Box::new(Term::Add(t, rhs)),
                    body: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Var("".to_string()))))))
                },
                _ => match *rhs {
                    Term::Zero => Term::Return(lhs),
                    Term::Succ(t) => Term::Bind {
                        var: "".to_string(),
                        val: Box::new(Term::Add(lhs, t)),
                        body: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Var("".to_string()))))))
                    },
                    _ => Term::Add(lhs, rhs)
                }
            },
            Term::Bind { var, val, body } => match *body {
                Term::Fail => Term::Fail,
                body => match val.propogate() {
                    Term::Fail => Term::Fail,
                    Term::Return(t) => body.substitute(&var, &t),
                    Term::Bind { var: var2, val: val2, body: body2 } => match *body2 {
                        Term::Return(t) => Term::Bind { var: var2, val: val2, body: Box::new(body.substitute(&var, &t)) },
                        _ => Term::Bind { var, val: Box::new(Term::Bind { var: var2, val: val2, body: body2 }), body: Box::new(body.propogate()) }
                    }
                    val => Term::Bind { var, val: Box::new(val), body: Box::new(body.propogate()) }
                }
            },
            Term::Force(t) => match *t {
                Term::Thunk(t) => *t,
                _ => Term::Force(t)
            },
            Term::App(lhs, rhs) => match lhs.propogate() {
                Term::Lambda { mut args, body } => {
                    let var = args.remove(args.len() - 1);
                    let body = body.substitute(var, &rhs);

                    if args.len() == 0 {
                        body
                    } else {
                        Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                            args,
                            body: Box::new(body)
                        }))))
                    }
                },
                lhs => Term::App(Box::new(lhs), rhs)
            },
            Term::Exists { var, r#type, body } => match body.propogate() {
                Term::Fail => Term::Fail,
                body => Term::Exists { var, r#type, body: Box::new(body) }
            },
            Term::Equate { lhs, rhs, body } => match *lhs {
                Term::Zero => match *rhs {
                    Term::Succ(_) => Term::Fail,
                    _ => Term::Equate { lhs, rhs, body }
                },
                Term::Succ(t1) => match *rhs {
                    Term::Zero => Term::Fail,
                    Term::Succ(t2) => Term::Equate { lhs: t1, rhs: t2, body },
                    _ => Term::Equate { lhs: Box::new(Term::Succ(t1)), rhs, body }
                },
                _ => Term::Equate { lhs, rhs, body }
            },
            Term::Choice(mut v) => if v.len() == 0 {
                Term::Fail
            } else if v.len() == 1 {
                v.remove(0)
            } else {
                Term::Choice(
                    v.into_iter()
                        .flat_map(|t| t.propogate_flat())
                        .filter(|t| *t != Term::Fail)
                        .collect()
                )
            },
            t => t
        }
    }

    fn propogate_flat(self) -> Vec<Term<'a>> {
        match self {
            Term::Choice(v) => v.into_iter().flat_map(|t| t.propogate_flat()).collect(),
            _ => vec![self.propogate()]
        }
    }

    pub fn substitute(self, var: &str, term: &Term<'a>) -> Term<'a> {
        match self {
            Term::Var(s) => if s == var { term.clone() } else { Term::Var(s) },
            Term::Succ(t) => Term::Succ(Box::new(t.substitute(var, term))),
            Term::Eq(lhs, rhs) => Term::Eq(
                Box::new(lhs.substitute(var, term)),
                Box::new(rhs.substitute(var, term))
            ),
            Term::NEq(lhs, rhs) => Term::NEq(
                Box::new(lhs.substitute(var, term)),
                Box::new(rhs.substitute(var, term))
            ),
            Term::Not(t) => Term::Not(Box::new(t.substitute(var, term))),
            Term::If { cond, then, r#else } => Term::If {
                cond: Box::new(cond.substitute(var, term)),
                then: Box::new(then.substitute(var, term)),
                r#else: Box::new(r#else.substitute(var, term))
            },
            Term::Bind { var: v, val, body } => {
                let flag = v == var;

                Term::Bind {
                    var: v,
                    val: Box::new(val.substitute(var, term)),
                    body: if flag { body } else {
                        Box::new(body.substitute(var, term))
                    }
                }
            },
            Term::Exists { var: v, r#type, body } => {
                Term::Exists {
                    var: v,
                    r#type,
                    body: if v == var { body } else {
                        Box::new(body.substitute(var, term))
                    }
                }
            },
            Term::Equate { lhs, rhs, body } => eval_equate(
                lhs.substitute(var, term),
                rhs.substitute(var, term),
                body.substitute(var, term)
            ),
            Term::Lambda { args, body } => {
                let flag = args.contains(&var);

                Term::Lambda {
                    args,
                    body: if flag { 
                        body
                    } else {
                        Box::new(body.substitute(var, term))
                    }
                }
            },
            Term::Choice(c) => Term::Choice(
                c.into_iter()
                    .map(|t| t.substitute(var, term))
                    .collect()
            ),
            Term::Thunk(t) => Term::Thunk(Box::new(t.substitute(var, term))),
            Term::Return(t) => Term::Return(Box::new(t.substitute(var, term))),
            Term::Force(t) => Term::Force(Box::new(t.substitute(var, term))),
            Term::Add(lhs, rhs) => Term::Add(
                Box::new(lhs.substitute(var, term)),
                Box::new(rhs.substitute(var, term))
            ),
            Term::App(lhs, rhs) => Term::App(
                Box::new(lhs.substitute(var, term)),
                Box::new(rhs.substitute(var, term))
            ),
            _ => self
        }
    }

    // includes no successors
    pub fn is_succ_of(&self, var: &str) -> bool {
        match self {
            Term::Var(v) => v == var,
            Term::Succ(t) => t.is_succ_of(var),
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let term = Term::Bind {
            var: "0".to_string(),
            val: Box::new(Term::Return(Box::new(Term::Zero))),
            body: Box::new(Term::Bind {
                var: "1".to_string(),
                val: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                    args: vec!["x"],
                    body: Box::new(Term::Return(Box::new(Term::Var("x".to_string()))))
                }))))),
                body: Box::new(Term::App(
                    Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                    Box::new(Term::Var("0".to_string()))
                ))
            })
        };

        assert_eq!(
            term.propogate(),
            Term::Return(Box::new(Term::Zero))
        );
    }

    #[test]
    fn test2() {
        let term = Term::Bind {
            var: "0".to_string(),
            val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Var("n".to_string())))))),
            body: Box::new(Term::Bind {
                var: "1".to_string(),
                val: Box::new(Term::Return(Box::new(Term::Zero))),
                body: Box::new(Term::Bind {
                    var: "2".to_string(),
                    val: Box::new(Term::Add(Box::new(Term::Var("0".to_string())), Box::new(Term::Var("1".to_string())))),
                    body: Box::new(Term::Equate {
                        lhs: Box::new(Term::Var("2".to_string())),
                        rhs: Box::new(Term::Succ(Box::new(Term::Zero))),
                        body: Box::new(Term::Return(Box::new(Term::Zero)))
                    })
                })
            })
        };

        assert_eq!(
            term.propogate(),
            Term::Equate {
                lhs: Box::new(Term::Var("n".to_string())),
                rhs: Box::new(Term::Zero),
                body: Box::new(Term::Return(Box::new(Term::Zero)))
            }
        );
    }

    #[test]
    fn test3() {
        let term = Term::Exists {
            var: "n",
            r#type: Type::Ident("Nat"),
            body: Box::new(Term::Equate {
                lhs: Box::new(Term::Succ(Box::new(Term::Var("n".to_string())))),
                rhs: Box::new(Term::Zero),
                body: Box::new(Term::Return(Box::new(Term::Zero)))
            })
        };

        assert_eq!(
            term.propogate(),
            Term::Fail
        );
    }

    #[test]
    fn test4() {
        let term = Term::Exists {
            var: "n",
            r#type: Type::Ident("Nat"),
            body: Box::new(Term::Bind {
                var: "1".to_string(),
                val: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                    args: vec!["x"],
                    body: Box::new(Term::Return(Box::new(Term::Var("x".to_string()))))
                }))))),
                body: Box::new(Term::Bind {
                    var: "2".to_string(),
                    val: Box::new(Term::App(
                        Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                        Box::new(Term::Var("n".to_string()))
                    )),
                    body: Box::new(Term::Equate {
                        lhs: Box::new(Term::Var("2".to_string())),
                        rhs: Box::new(Term::Zero),
                        body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
                    })
                })
            })
        };

        assert_eq!(
            term.propogate(),
            Term::Exists {
                var: "n",
                r#type: Type::Ident("Nat"),
                body: Box::new(Term::Equate {
                    lhs: Box::new(Term::Var("n".to_string())),
                    rhs: Box::new(Term::Zero),
                    body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
                })
            }
        );
    }

    #[test]
    fn test5() {
        let term = Term::Bind {
            var: "0".to_string(),
            val: Box::new(Term::Add(Box::new(Term::Var("n".to_string())), Box::new(Term::Var("n".to_string())))),
            body: Box::new(Term::Bind {
                var: "1".to_string(),
                val: Box::new(Term::Return(Box::new(Term::Zero))),
                body: Box::new(Term::Fail)
            })
        };

        assert_eq!(
            term.propogate(),
            Term::Fail
        );
    }

    #[test]
    fn test6() {
        let term = Term::Choice(vec![
            Term::Equate { lhs: Box::new(Term::Zero), rhs: Box::new(Term::Succ(Box::new(Term::Zero))), body: Box::new(Term::Zero) }
        ]);

        assert_eq!(
            term.propogate(),
            Term::Fail
        );
    }

    #[test]
    fn test7() {
        let term = Term::Bind {
            var: "0".to_string(),
            val: Box::new(Term::Equate {
                lhs: Box::new(Term::Succ(Box::new(Term::Var("n".to_string())))),
                rhs: Box::new(Term::Zero),
                body: Box::new(Term::Var("n".to_string()))
            }),
            body: Box::new(Term::Equate {
                lhs: Box::new(Term::Var("n".to_string())),
                rhs: Box::new(Term::Zero),
                body: Box::new(Term::Zero)
            })
        };

        assert_eq!(
            term.propogate(),
            Term::Fail
        )
    }
}