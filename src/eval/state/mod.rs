use std::{borrow::BorrowMut, collections::HashMap};

use env::{Env, EnvValue};
use stack::{Stack, StackTerm};

use crate::cbpv::Term;

mod env;
mod stack;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    env: Env,
    term: StateTerm,
    stack: Stack
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum StateTerm {
    Term(Term),
    Closure(Box<State>)
}

impl State {
    pub fn new(mut cbpv: HashMap<String, Term>) -> Self {
        let term = cbpv.remove("main").unwrap();

        let mut env = Env::new();
        cbpv.into_iter()
            .for_each(|(var, val)| {
                env.store(var, val);
            });

        let mut stack = Stack::new();
        stack.push(StackTerm::PopEnv);

        State {
            env: Env::push(env), term: StateTerm::Term(term), stack
        }
    }

    pub fn step(mut self) -> Vec<State> {
        match self.term.clone() {
            StateTerm::Term(term) => match term {
                Term::Return(val) => {
                    let val = match *val {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => term,
                            EnvValue::Type(_) => todo!(),
                            EnvValue::Closure(_, _) => unreachable!()
                        },
                        _ => *val
                    };

                    match self.stack.pop() {
                        Some(term) => match term {
                            StackTerm::Cont(var, term) => {
                                let mut env = if self.env.in_scope(&var) {
                                    self.stack.push(StackTerm::PopEnv);
                                    Env::push(self.env)
                                } else {
                                    self.env
                                };

                                env.borrow_mut().store(var, val);

                                vec![State {
                                    env, term: StateTerm::Term(term), stack: self.stack
                                }]
                            },
                            StackTerm::PopEnv => vec![State {
                                env: *self.env.pop().unwrap(),
                                term: StateTerm::Term(Term::Return(Box::new(val))),
                                stack: self.stack
                            }],
                            _ => unreachable!()
                        },
                        None => vec![State {
                            env: self.env,
                            term: StateTerm::Term(Term::Return(Box::new(val))),
                            stack: self.stack
                        }]
                    }
                },
                Term::Bind { var, val, body } => {
                    self.stack.push(StackTerm::Cont(var, *body));

                    vec![State {
                        env: self.env, term: StateTerm::Term(*val), stack: self.stack
                    }]
                },
                Term::Add(lhs, rhs) => {
                    let lhs = match *lhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => term,
                            EnvValue::Type(_) => todo!(),
                            EnvValue::Closure(_, _) => unreachable!()
                        },
                        _ => *lhs
                    };

                    let rhs = match *rhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => term,
                            EnvValue::Type(_) => todo!(),
                            EnvValue::Closure(_, _) => unreachable!()
                        },
                        _ => *rhs
                    };

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(add_terms(lhs, rhs)),
                        stack: self.stack
                    }]
                },
                Term::Force(term) => {
                    match *term {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                Term::Thunk(term) => vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(*term),
                                    stack: self.stack
                                }],
                                _ => unreachable!()
                            },
                            EnvValue::Closure(term, env) => match term {
                                Term::Thunk(term) => vec![State {
                                    env: self.env,
                                    term: StateTerm::Closure(Box::new(State {
                                        env, term: StateTerm::Term(*term), stack: Stack::new()
                                    })),
                                    stack: self.stack
                                }],
                                _ => unreachable!()
                            }
                            _ => unreachable!()
                        },
                        Term::Thunk(term) => vec![State {
                            env: self.env,
                            term: StateTerm::Term(*term),
                            stack: self.stack
                        }],
                        _ => unreachable!()
                    }
                },
                Term::App(lhs, rhs) => {
                    self.stack.push(StackTerm::Term(*rhs));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(*lhs),
                        stack: self.stack
                    }]
                },
                Term::Lambda { var, body } => match self.stack.pop() {
                    Some(term) => match term {
                        StackTerm::Term(term) => {
                            let term = match term {
                                Term::Var(var) => match self.env.lookup(&var).unwrap() {
                                    EnvValue::Term(term) => term,
                                    EnvValue::Type(_) => todo!(),
                                    EnvValue::Closure(_, _) => unreachable!()
                                },
                                _ => term
                            };

                            let env = self.env.clone();
                            let mut env = if env.in_scope(&var) {
                                Env::push(env)
                            } else {
                                env
                            };

                            env.store(var, term);

                            vec![State {
                                env: self.env,
                                term: StateTerm::Closure(Box::new(State {
                                    env, term: StateTerm::Term(*body), stack: Stack::new()
                                })),
                                stack: self.stack
                            }]
                        },
                        _ => unreachable!()
                    },
                    _ => unreachable!()
                },
                _ => vec![State {
                    env: self.env,
                    term: self.term,
                    stack: self.stack
                }]
            },
            StateTerm::Closure(mut state) => match state.term {
                StateTerm::Term(term) => match term {
                    Term::Return(term) => match *term {
                        Term::Var(var) => match state.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => vec![State {
                                env: self.env,
                                term: StateTerm::Term(term),
                                stack: self.stack
                            }],
                            EnvValue::Type(_) => todo!(),
                            EnvValue::Closure(term, env) => vec![State {
                                env: self.env,
                                term: StateTerm::Closure(Box::new(State {
                                    env,
                                    term: StateTerm::Term(term),
                                    stack: Stack::new()
                                })),
                                stack: self.stack
                            }]
                        },
                        Term::Thunk(_) => match self.stack.pop() {
                            Some(t) => match t {
                                StackTerm::Cont(var, body) => {
                                    let mut env = if self.env.in_scope(&var) {
                                        self.stack.push(StackTerm::PopEnv);
                                        Env::push(self.env)
                                    } else {
                                        self.env
                                    };

                                    env.store_closure(var, *term, state.env);

                                    vec![State {
                                        env,
                                        term: StateTerm::Term(body),
                                        stack: self.stack
                                    }]
                                },
                                StackTerm::PopEnv => vec![State {
                                    env: *self.env.pop().unwrap(),
                                    term: StateTerm::Closure(Box::new(State {
                                        env: state.env,
                                        term: StateTerm::Term(Term::Return(term)),
                                        stack: state.stack
                                    })),
                                    stack: self.stack
                                }],
                                _ => unreachable!()
                            },
                            None => unreachable!()
                        },
                        term => vec![State {
                            env: self.env,
                            term: StateTerm::Term(term),
                            stack: self.stack
                        }]
                    },
                    Term::Lambda { var, body } => match self.stack.pop().unwrap() {
                        StackTerm::Term(term) => {
                            let mut env = if state.env.in_scope(&var) {
                                state.stack.push(StackTerm::PopEnv);
                                Env::push(state.env)
                            } else {
                                state.env
                            };

                            env.store(var, term);

                            vec![State {
                                env: self.env,
                                term: StateTerm::Closure(Box::new(State {
                                    env,
                                    term: StateTerm::Term(*body),
                                    stack: state.stack
                                })),
                                stack: self.stack
                            }]
                        },
                        _ => unreachable!()
                    },
                    _ => {
                        let states = State {
                            env: state.env,
                            term: StateTerm::Term(term),
                            stack: state.stack
                        }.step();

                        states.into_iter()
                            .map(|state| State {
                                env: self.env.clone(),
                                term: StateTerm::Closure(Box::new(state)),
                                stack: self.stack.clone()
                            }).collect()
                    }
                },
                _ => state.step().into_iter()
                    .map(|state| State {
                        env: self.env.clone(),
                        term: StateTerm::Closure(Box::new(state)),
                        stack: self.stack.clone()
                    }).collect()
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
