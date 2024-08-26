use std::collections::HashMap;

use closure::{Closure, ClosureVars};
use env::{env::Env, env_value::EnvValue};
use stack::{Stack, StackTerm};
use state_term::StateTerm;

use crate::cbpv::Term;

mod state_term;
mod env;
mod stack;
mod closure;

#[derive(Debug)]
pub struct State {
    env: Env,
    term: StateTerm,
    stack: Stack
}

impl State {
    pub fn new(mut cbpv: HashMap<String, Term>) -> Self {
        let term = cbpv.remove("main").unwrap();

        let mut env = Env::new();
        cbpv.into_iter()
            .for_each(|(var, val)| {
                env.store(var, StateTerm::Term(val))
            });

        State {
            env,
            term: StateTerm::Term(term),
            stack: Stack::new()
        }
    }

    pub fn step(mut self) -> Vec<State> {
        match self.term {
            StateTerm::Term(term) => match term {
                Term::Return(term) => match *term {
                    Term::Var(var) => match self.env.lookup(&var).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => vec![State {
                                env: self.env,
                                term: StateTerm::Term(Term::Return(Box::new(term))),
                                stack: self.stack
                            }],
                            StateTerm::Closure(closure) => vec![State {
                                env: self.env,
                                term: StateTerm::Closure(Closure {
                                    term: Term::Return(Box::new(closure.term)),
                                    vars: closure.vars
                                }),
                                stack: self.stack
                            }]
                        },
                        EnvValue::Type(_) => todo!()
                    },
                    _ => match self.stack.pop() {
                        Some(s) => match s {
                            StackTerm::Cont(var, body) => match body {
                                StateTerm::Term(_) => {
                                    self.env.store(var.clone(), StateTerm::Term(*term));
                                    self.stack.push(StackTerm::Release(var));

                                    vec![State {
                                        env: self.env,
                                        term: body,
                                        stack: self.stack
                                    }]
                                },
                                StateTerm::Closure(mut body) => {
                                    body.vars.store(var, EnvValue::Term(StateTerm::Term(*term)));

                                    vec![State {
                                        env: self.env,
                                        term: StateTerm::Closure(body),
                                        stack: self.stack
                                    }]
                                }
                            },
                            StackTerm::Release(var) => {
                                self.env.release(&var);

                                vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(Term::Return(term)),
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
                    }
                }
                Term::Bind { var, val, body } => {
                    self.stack.push(StackTerm::Cont(var, StateTerm::Term(*body)));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(*val),
                        stack: self.stack
                    }]
                },
                Term::App(lhs, rhs) => {
                    let rhs = match *rhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => term,
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    self.stack.push(StackTerm::Term(rhs));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(*lhs),
                        stack: self.stack
                    }]
                },
                Term::Force(term) => match *term {
                    Term::Var(var) => match self.env.lookup(&var).unwrap() {
                        EnvValue::Term(term) => match term {
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
                                    term: StateTerm::Closure(Closure {
                                        term: *term, vars: closure.vars
                                    }),
                                    stack: self.stack
                                }],
                                _ => unreachable!()
                            }
                        },
                        EnvValue::Type(_) => unreachable!()
                    },
                    _ => unreachable!()
                },
                Term::Lambda { var, free_vars, body } => match self.stack.pop().unwrap() {
                    StackTerm::Term(term) => {
                        let mut vars = ClosureVars::new();
                        vars.store(var, EnvValue::Term(term));

                        free_vars.into_iter()
                            .for_each(|var| {
                                let val = self.env.lookup(&var).unwrap();

                                vars.store(var, val);
                            });

                        vec![State {
                            env: self.env,
                            term: StateTerm::Closure(Closure {
                                term: *body, vars
                            }),
                            stack: self.stack
                        }]
                    },
                    _ => unreachable!()
                }
                _ => unreachable!()
            },
            StateTerm::Closure(closure) => match closure.term {
                Term::Return(term) => match *term {
                    Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => vec![State {
                                env: self.env,
                                term: StateTerm::Term(Term::Return(Box::new(term))),
                                stack: self.stack
                            }],
                            StateTerm::Closure(closure) => vec![State {
                                env: self.env,
                                term: StateTerm::Closure(Closure {
                                    term: Term::Return(Box::new(closure.term)), vars: closure.vars
                                }),
                                stack: self.stack
                            }]
                        },
                        EnvValue::Type(_) => todo!()
                    },
                    _ => match self.stack.pop() {
                        Some(s) => match s {
                            StackTerm::Cont(var, body) => match body {
                                StateTerm::Term(_) => {
                                    self.env.store(var.clone(), StateTerm::Closure(Closure {
                                        term: *term, vars: closure.vars
                                    }));

                                    self.stack.push(StackTerm::Release(var));

                                    vec![State {
                                        env: self.env,
                                        term: body,
                                        stack: self.stack
                                    }]
                                },
                                StateTerm::Closure(mut body) => {
                                    body.vars.store(var, EnvValue::Term(StateTerm::Closure(Closure {
                                        term: *term, vars: closure.vars
                                    })));

                                    vec![State {
                                        env: self.env,
                                        term: StateTerm::Closure(body),
                                        stack: self.stack
                                    }]
                                }
                            },
                            StackTerm::Release(var) => {
                                self.env.release(&var);

                                vec![State {
                                    env: self.env,
                                    term: StateTerm::Closure(Closure {
                                        term: Term::Return(term), vars: closure.vars
                                    }),
                                    stack: self.stack
                                }]
                            },
                            StackTerm::Term(_) => unreachable!()
                        },
                        None => unreachable!()
                    }
                }
                Term::Bind { var, val, body } => {
                    self.stack.push(StackTerm::Cont(var, StateTerm::Closure(Closure {
                        term: *body, vars: closure.vars.clone()
                    })));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term: *val, vars: closure.vars
                        }),
                        stack: self.stack
                    }]
                }
                Term::App(lhs, rhs) => {
                    let rhs = match *rhs {
                        Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                            EnvValue::Term(term) => term,
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    self.stack.push(StackTerm::Term(rhs));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term: *lhs, vars: closure.vars
                        }),
                        stack: self.stack
                    }]
                }
                Term::Force(term) => match *term {
                    Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                        EnvValue::Term(term) => match term {
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
                                    term: StateTerm::Closure(Closure {
                                        term: *term, vars: closure.vars
                                    }),
                                    stack: self.stack
                                }],
                                _ => unreachable!()
                            }
                        },
                        EnvValue::Type(_) => unreachable!()
                    },
                    _ => unreachable!()
                },
                Term::Lambda { var, free_vars,  body } => match self.stack.pop().unwrap() {
                    StackTerm::Term(term) => {
                        let mut vars = ClosureVars::new();
                        vars.store(var, EnvValue::Term(term));

                        free_vars.iter()
                            .for_each(|var| {
                                let val = closure.vars.lookup(var).unwrap();

                                vars.store(var.clone(), val);
                            });

                        vec![State {
                            env: self.env,
                            term: StateTerm::Closure(Closure {
                                term: *body, vars
                            }),
                            stack: self.stack
                        }]
                    },
                    _ => unreachable!()
                }
                _ => unreachable!()
            }
        }
    }

    pub fn is_value(&self) -> bool {
        if self.stack.is_empty() {
            match &self.term {
                StateTerm::Term(term) => match term {
                    Term::Return(term) => match **term {
                        Term::Var(_) => false,
                        _ => true
                    },
                    _ => false
                },
                _ => false
            }
        } else {
            false
        }
    }

    pub fn as_term(self) -> Term {
        match self.term {
            StateTerm::Term(term) => term,
            StateTerm::Closure(_) => unreachable!()
        }
    }
}
