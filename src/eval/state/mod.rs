use std::collections::HashMap;

use closure::{Closure, ClosureVars};
use env::{env::Env, env_value::EnvValue};
use env_lookup::EnvLookup;
use stack::{Stack, StackTerm};
use state_term::StateTerm;

use crate::cbpv::Term;

mod state_term;
mod env;
mod env_lookup;
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
                Term::Return(term) => {
                    let val = expand_value(*term, &self.env);

                    match self.stack.pop() {
                        Some(s) => match s {
                            StackTerm::Cont(var, body) => match body {
                                StateTerm::Term(_) => {
                                    self.env.store(var.clone(), val);
                                    self.stack.push(StackTerm::Release(var));

                                    vec![State {
                                        env: self.env,
                                        term: body,
                                        stack: self.stack
                                    }]
                                },
                                StateTerm::Closure(mut body) => {
                                    body.vars.store(var, val);

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
                                    term: match val {
                                        StateTerm::Term(term) => StateTerm::Term(Term::Return(Box::new(term))),
                                        StateTerm::Closure(closure) => StateTerm::Closure(Closure {
                                            term: Term::Return(Box::new(closure.term)), vars: closure.vars
                                        })
                                    },
                                    stack: self.stack
                                }]
                            },
                            StackTerm::Term(_) => unreachable!()
                        },
                        None => vec![State {
                            env: self.env,
                            term: match val {
                                StateTerm::Term(term) => StateTerm::Term(Term::Return(Box::new(term))),
                                StateTerm::Closure(closure) => StateTerm::Closure(Closure {
                                    term: Term::Return(Box::new(closure.term)), vars: closure.vars
                                })
                            },
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
                Term::Add(lhs, rhs) => {
                    let lhs = match *lhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => unreachable!()
                        },
                        _ => unreachable!()
                    };

                    let rhs = match *rhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => unreachable!()
                        },
                        _ => unreachable!()
                    };

                    let term = add_terms(lhs, rhs);

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(term),
                        stack: self.stack
                    }]
                },
                Term::Eq(lhs, rhs) => {
                    let lhs = match *lhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    let rhs = match *rhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(Term::Return(Box::new(Term::Bool(lhs == rhs)))),
                        stack: self.stack
                    }]
                },
                Term::NEq(lhs, rhs) => {
                    let lhs = match *lhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    let rhs = match *rhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(Term::Return(Box::new(Term::Bool(lhs != rhs)))),
                        stack: self.stack
                    }]
                },
                Term::Not(term) => match *term {
                    Term::Var(var) => match self.env.lookup(&var).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => match term {
                                Term::Bool(bool) => vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(Term::Return(Box::new(Term::Bool(!bool)))),
                                    stack: self.stack
                                }],
                                _ => unreachable!()
                            },
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => unreachable!()
                    },
                    _ => unreachable!()
                },
                Term::If { cond, then, r#else } => {
                    let term = match *cond {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => match term {
                                    Term::Bool(bool) => if bool { *then } else { *r#else },
                                    _ => unreachable!()
                                },
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => unreachable!()
                        },
                        _ => unreachable!()
                    };

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(term),
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
                        vars.store(var, term);

                        free_vars.into_iter()
                            .for_each(|var| {
                                match self.env.lookup(&var).unwrap() {
                                    EnvValue::Term(term) => vars.store(var, term),
                                    EnvValue::Type(_) => todo!()
                                }
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
                },
                Term::Choice(choices) => choices.into_iter()
                    .map(|choice| State {
                        env: self.env.clone(),
                        term: StateTerm::Term(choice),
                        stack: self.stack.clone()
                    }).collect(),
                _ => unreachable!()
            },
            StateTerm::Closure(closure) => match closure.term {
                Term::Return(term) => {
                    let val = expand_closure_value(*term, closure.vars);

                    match self.stack.pop() {
                        Some(s) => match s {
                            StackTerm::Cont(var, body) => match body {
                                StateTerm::Term(_) => {
                                    self.env.store(var.clone(), val);

                                    self.stack.push(StackTerm::Release(var));

                                    vec![State {
                                        env: self.env,
                                        term: body,
                                        stack: self.stack
                                    }]
                                },
                                StateTerm::Closure(mut body) => {
                                    body.vars.store(var, val);

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
                                    term: match val {
                                        StateTerm::Term(term) => StateTerm::Term(Term::Return(Box::new(term))),
                                        StateTerm::Closure(val) => StateTerm::Closure(Closure {
                                            term: Term::Return(Box::new(val.term)), vars: val.vars
                                        })
                                    },
                                    stack: self.stack
                                }]
                            },
                            StackTerm::Term(_) => unreachable!()
                        },
                        None => unreachable!()
                    }
                },
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
                },
                Term::Add(lhs, rhs) => {
                    let lhs = match *lhs {
                        Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => unreachable!()
                        },
                        _ => unreachable!()
                    };

                    let rhs = match *rhs {
                        Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => unreachable!()
                        },
                        _ => unreachable!()
                    };

                    let term = add_terms(lhs, rhs);

                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term, vars: closure.vars
                        }),
                        stack: self.stack
                    }]
                },
                Term::Eq(lhs, rhs) => {
                    let lhs = match *lhs {
                        Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    let rhs = match *rhs {
                        Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(Term::Return(Box::new(Term::Bool(lhs == rhs)))),
                        stack: self.stack
                    }]
                },
                Term::NEq(lhs, rhs) => {
                    let lhs = match *lhs {
                        Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    let rhs = match *rhs {
                        Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => todo!()
                        },
                        _ => unreachable!()
                    };

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(Term::Return(Box::new(Term::Bool(lhs != rhs)))),
                        stack: self.stack
                    }]
                },
                Term::Not(term) => match *term {
                    Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => match term {
                                Term::Bool(bool) => vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(Term::Return(Box::new(Term::Bool(!bool)))),
                                    stack: self.stack
                                }],
                                _ => unreachable!()
                            },
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => unreachable!()
                    },
                    _ => unreachable!()
                },
                Term::If { cond, then, r#else } => {
                    let term = match *cond {
                        Term::Var(var) => match closure.vars.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => match term {
                                    Term::Bool(bool) => if bool { *then } else { *r#else },
                                    _ => unreachable!()
                                },
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => unreachable!()
                        },
                        _ => unreachable!()
                    };

                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term, vars: closure.vars
                        }),
                        stack: self.stack
                    }]
                },
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
                },
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
                        vars.store(var, term);

                        free_vars.into_iter()
                            .for_each(|var| {
                                match closure.vars.lookup(&var).unwrap() {
                                    EnvValue::Term(term) => vars.store(var, term),
                                    EnvValue::Type(_) => todo!()
                                }
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
                },
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

fn expand_value(term: Term, env: &impl EnvLookup) -> StateTerm {
    match term {
        Term::Zero => StateTerm::Term(term),
        Term::Bool(bool) => StateTerm::Term(Term::Bool(bool)),
        Term::Var(var) => match env.lookup(&var).unwrap() {
            EnvValue::Term(term) => term,
            EnvValue::Type(_) => todo!()
        },
        Term::Succ(term) => match expand_value(*term, env) {
            StateTerm::Term(term) => StateTerm::Term(Term::Succ(Box::new(term))),
            StateTerm::Closure(closure) => StateTerm::Closure(Closure {
                term: Term::Succ(Box::new(closure.term)), vars: closure.vars
            })
        },
        _ => unreachable!()
    }
}

fn expand_closure_value(term: Term, vars: ClosureVars) -> StateTerm {
    match term {
        Term::Thunk(_) => StateTerm::Closure(Closure {
            term, vars
        }),
        _ => expand_value(term, &vars)
    }
}

fn add_terms(lhs: Term, rhs: Term) -> Term {
    match lhs {
        Term::Bool(bool) => Term::Bool(bool),
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