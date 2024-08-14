use std::collections::HashMap;

use frame::{Frame, LookupResult};
use stack::{Stack, StackTerm};

use crate::{cbpv::Term, parser::syntax::r#type::Type};

mod frame;
mod stack;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    frame: Frame,
    term: Term,
    stack: Stack
}

impl State {
    pub fn new(mut cbpv: HashMap<String, Term>) -> Self {
        let term = cbpv.remove("main").unwrap();

        let mut frame = Frame::new();
        cbpv.into_iter()
            .for_each(|(var, val)| {
                frame.store(var, val);
            });

        let stack = Stack::new();

        State {
            frame, term, stack
        }
    }

    pub fn step(mut self) -> Vec<State> {
        match self.term.clone() {
            Term::Return(val) => {
                let val = self.lookup_value(*val);

                match self.stack.pop() {
                    Some(term) => match term {
                        StackTerm::Cont(var, term) => {
                            let mut frame = Frame::push(self.frame);
                            frame.store(var, val);

                            vec![State {
                                frame, term, stack: self.stack
                            }]
                        },
                        _ => unreachable!()
                    },
                    None => vec![State {
                        frame: self.frame.pop(),
                        term: Term::Return(Box::new(val)),
                        stack: self.stack
                    }]
                }
            },
            Term::Bind { var, val, body } => {
                self.stack.push(StackTerm::Cont(var, *body));

                vec![State {
                    frame: self.frame, term: *val, stack: self.stack
                }]
            },
            Term::Add(lhs, rhs) => {
                let lhs = self.lookup_value(*lhs);
                let rhs = self.lookup_value(*rhs);

                vec![State {
                    frame: self.frame.pop().pop(),
                    term: add_terms(lhs, rhs),
                    stack: self.stack
                }]
            },
            Term::Eq(lhs, rhs) => {
                let lhs = self.lookup_value(*lhs);
                let rhs = self.lookup_value(*rhs);

                vec![State {
                    frame: self.frame,
                    term: Term::Bool(if lhs == rhs { true } else { false }),
                    stack: self.stack
                }]
            },
            Term::NEq(lhs, rhs) => {
                let lhs = self.lookup_value(*lhs);
                let rhs = self.lookup_value(*rhs);

                vec![State {
                    frame: self.frame,
                    term: Term::Bool(if lhs == rhs { false } else { true }),
                    stack: self.stack
                }]
            },
            Term::Not(term) => {
                let term = self.lookup_value(*term);

                vec![State {
                    frame: self.frame,
                    term: Term::Bool(match term {
                        Term::Bool(bool) => !bool,
                        _ => unreachable!()
                    }),
                    stack: self.stack
                }]
            },
            Term::If { cond, then, r#else } => match self.lookup_value(*cond) {
                Term::Bool(bool) => vec![State {
                    frame: self.frame,
                    term: if bool { *then } else { *r#else },
                    stack: self.stack
                }],
                _ => unreachable!()
            },
            Term::Choice(_) => todo!(),
            Term::Force(term) => {
                let term = match *term {
                    Term::Var(var) => match self.frame.lookup(&var) {
                        LookupResult::Term(term) => term,
                        _ => unreachable!()
                    },
                    _ => *term
                };

                match term {
                    Term::Thunk(term) => vec![State {
                        frame: self.frame,
                        term: *term,
                        stack: self.stack
                    }],
                    _ => unreachable!()
                }
            },
            Term::App(lhs, rhs) => {
                self.stack.push(StackTerm::Term(*rhs));

                vec![State {
                    frame: self.frame,
                    term: *lhs,
                    stack: self.stack
                }]
            },
            Term::Lambda { var, body } => match self.stack.pop() {
                Some(term) => match term {
                    StackTerm::Term(term) => vec![State {
                        frame: self.frame,
                        term: Term::Bind { var, val: Box::new(term), body },
                        stack: self.stack
                    }],
                    _ => unreachable!()
                },
                _ => unreachable!()
            },
            Term::Exists { var, r#type, body } => {
                let mut frame = Frame::push(self.frame);
                frame.bind(var, r#type);
                
                vec![State {
                    frame, term: *body, stack: self.stack
                }]
            },
            Term::Equate { lhs, rhs, body } => todo!(),
            _ => vec![State {
                frame: self.frame,
                term: self.term,
                stack: self.stack
            }]
        }
    }

    pub fn as_term(self) -> Term {
        self.term
    }

    fn lookup_value(&self, term: Term) -> Term {
        match term {
            Term::Var(var) => match self.frame.lookup(&var) {
                LookupResult::Term(term) => term,
                LookupResult::Type(_) => todo!()
            },
            Term::Succ(term) => Term::Succ(Box::new(self.lookup_value(*term))),
            _ => term
        }
    }
}

fn add_terms(lhs: Term, rhs: Term) -> Term {
    match lhs {
        Term::Zero => Term::Return(Box::new(rhs)),
        Term::Succ(lhs) => match rhs {
            Term::Zero => Term::Return(Box::new(Term::Succ(lhs))),
            Term::Succ(rhs) => Term::Bind {
                var: "0".to_string(),
                val: Box::new(Term::Return(lhs)),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Return(rhs)),
                    body: Box::new(Term::Bind {
                        var: "2".to_string(),
                        val: Box::new(Term::Add(Box::new(Term::Var("0".to_string())), Box::new(Term::Var("1".to_string())))),
                        body: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Var("2".to_string()))))))))
                    })
                })
            },
            _ => unreachable!()
        },
        _ => unreachable!()
    }
}
