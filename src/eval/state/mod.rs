use std::{cell::RefCell, collections::HashMap, rc::Rc};

use closure::Closure;
use env::{Env, EnvValue};
use stack::{Stack, StackTerm};
use state_term::StateTerm;

use crate::cbpv::Term;

mod state_term;
mod env;
mod stack;
mod closure;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    env: Rc<RefCell<Env>>,
    term: StateTerm,
    stack: Stack
}

impl State {
    pub fn new(mut cbpv: HashMap<String, Term>) -> Self {
        let term = cbpv.remove("main").unwrap();

        let mut env = Env::new();
        cbpv.into_iter()
            .for_each(|(var, val)| {
                env.store(var, StateTerm::Term(val));
            });

        State {
            env: Rc::new(RefCell::new(env)),
            term: StateTerm::Term(term),
            stack: Stack::new()
        }
    }

    pub fn step(mut self) -> Vec<State> {
        match self.term {
            StateTerm::Term(term) => match term {
                Term::Return(term) => match self.stack.pop() {
                    Some(s) => match s {
                        StackTerm::Cont(var, body) => {
                            let env = if self.env.borrow().in_scope(&var) {
                                self.stack.push(StackTerm::PopEnv);
                                Rc::new(RefCell::new(Env::push(&self.env)))
                            } else {
                                self.env
                            };

                            env.borrow_mut().store(var, StateTerm::Closure(Closure::new(
                                *term, &env
                            )));

                            vec![State {
                                env,
                                term: body,
                                stack: self.stack
                            }]
                        },
                        StackTerm::PopEnv => match *term {
                            Term::Var(var) => {
                                let term = self.env.borrow().lookup(&var).unwrap();

                                match term {
                                    EnvValue::Term(term) => match term {
                                        StateTerm::Term(term) => vec![State {
                                            env: self.env,
                                            term: StateTerm::Term(Term::Return(Box::new(term))),
                                            stack: self.stack
                                        }],
                                        StateTerm::Closure(closure) => vec![State {
                                            env: self.env,
                                            term: StateTerm::Closure(Closure::new(
                                                Term::Return(Box::new(closure.term)), &closure.env
                                            )),
                                            stack: self.stack
                                        }]
                                    },
                                    EnvValue::Type(_) => todo!()
                                }
                            },
                            term => vec![State {
                                env: self.env,
                                term: StateTerm::Term(Term::Return(Box::new(term))),
                                stack: self.stack
                            }]
                        },
                        StackTerm::Term(_) => unreachable!()
                    },
                    None => vec![State {
                        env: self.env,
                        term: StateTerm::Term(Term::Return(term)),
                        stack: self.stack
                    }]
                },
                Term::Bind { var, val, body } => {
                    self.stack.push(StackTerm::Cont(var, StateTerm::Term(*body)));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(*val),
                        stack: self.stack
                    }]
                },
                Term::App(lhs, rhs) => {
                    self.stack.push(StackTerm::Term(StateTerm::Term(*rhs)));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(*lhs),
                        stack: self.stack
                    }]
                },
                Term::Force(term) => {
                    let term = match *term {
                        Term::Var(var) => match self.env.borrow().lookup(&var).unwrap() {
                            EnvValue::Term(term) => term,
                            EnvValue::Type(_) => unreachable!()
                        },
                        _ => unreachable!()
                    };

                    match term {
                        StateTerm::Term(term) => match term {
                            Term::Thunk(term) => vec![State {
                                env: self.env,
                                term: StateTerm::Term(*term),
                                stack: self.stack
                            }],
                            _ => unreachable!()
                        },
                        StateTerm::Closure(closure) => match closure.term {
                            Term::Thunk(term) => vec![State {
                                env: self.env,
                                term: StateTerm::Closure(Closure::new(
                                    *term, &closure.env
                                )),
                                stack: self.stack
                            }],
                            _ => unreachable!()
                        }
                    }
                },
                _ => unreachable!()
            },
            StateTerm::Closure(closure) => match closure.term {
                Term::Return(term) => {
                    let term = match *term {
                        Term::Var(var) => match closure.env.borrow().lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => *term
                    };

                    match self.stack.pop() {
                        Some(s) => match s {
                            StackTerm::Cont(var, body) => match body {
                                StateTerm::Term(body) => {
                                    let env = if self.env.borrow().in_scope(&var) {
                                        self.stack.push(StackTerm::PopEnv);
                                        Rc::new(RefCell::new(Env::push(&self.env)))
                                    } else {
                                        self.env
                                    };

                                    env.borrow_mut().store(var, StateTerm::Closure(Closure::new(
                                        term, &closure.env
                                    )));

                                    vec![State {
                                        env,
                                        term: StateTerm::Term(body),
                                        stack: self.stack
                                    }]
                                },
                                StateTerm::Closure(body) => {
                                    let env = if body.env.borrow().in_scope(&var) {
                                        self.stack.push(StackTerm::PopEnv);
                                        Rc::new(RefCell::new(Env::push(&closure.env)))
                                    } else {
                                        body.env
                                    };

                                    env.borrow_mut().store(var, StateTerm::Closure(Closure::new(
                                        term, &closure.env
                                    )));

                                    vec![State {
                                        env: self.env,
                                        term: StateTerm::Closure(Closure::new(
                                            body.term, &env
                                        )),
                                        stack: self.stack
                                    }]
                                }
                            },
                            StackTerm::PopEnv => match closure.term {
                                Term::Var(var) => match closure.env.borrow().lookup(&var).unwrap() {
                                    EnvValue::Term(term) => match term {
                                        StateTerm::Term(term) => vec![State {
                                            env: self.env,
                                            term: StateTerm::Closure(Closure::new(
                                                Term::Return(Box::new(term)), &closure.env.borrow().pop().unwrap()
                                            )),
                                            stack: self.stack
                                        }],
                                        _ => todo!()
                                    },
                                    EnvValue::Type(_) => todo!()
                                },
                                _ => todo!()
                            }
                            StackTerm::Term(_) => unreachable!()
                        },
                        None => vec![State {
                            env: self.env,
                            term: StateTerm::Term(Term::Return(Box::new(term))),
                            stack: self.stack
                        }]
                    }
                },
                Term::Bind { var, val, body } => {
                    self.stack.push(StackTerm::Cont(var, StateTerm::Closure(Closure::new(
                        *body, &closure.env
                    ))));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure::new(
                            *val, &closure.env
                        )),
                        stack: self.stack
                    }]
                },
                Term::Add(lhs, rhs) => {
                    self.stack.push(StackTerm::Term(StateTerm::Closure(Closure::new(
                        *rhs, &closure.env
                    ))));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure::new(
                            *lhs, &closure.env
                        )),
                        stack: self.stack
                    }]
                },
                Term::Force(term) => {
                    let term = match *term {
                        Term::Var(var) => match closure.env.borrow().lookup(&var).unwrap() {
                            EnvValue::Term(term) => term,
                            EnvValue::Type(_) => unreachable!()
                        },
                        _ => unreachable!()
                    };

                    match term {
                        StateTerm::Term(term) => match term {
                            Term::Thunk(term) => vec![State {
                                env: self.env,
                                term: StateTerm::Closure(Closure::new(
                                    *term, &closure.env
                                )),
                                stack: self.stack
                            }],
                            _ => unreachable!()
                        },
                        _ => todo!()
                    }
                }
                _ => unreachable!()
            }
        }
    }

    pub fn as_term(self) -> Term {
        match self.term {
            StateTerm::Term(term) => term,
            StateTerm::Closure(_) => unreachable!()
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
