use std::{cell::RefCell, collections::HashMap, rc::Rc};

use closure::Closure;
use env::Env;
use equate::equate;
use stack::{Stack, StackTerm};
use state_term::StateTerm;

use crate::cbpv::{PMNat, PMNatSucc, Term, PM};

mod closure;
mod env;
mod equate;
mod stack;
mod state_term;

#[derive(Debug)]
pub struct State {
    env: Env, 
    term: StateTerm,
    stack: Stack
}

impl State {
    pub fn new(mut cbpv: HashMap<String, Term>) -> Self {
        let term = cbpv.remove("main").unwrap();

        let env = cbpv.into_iter()
            .fold(Env::new(), |mut env, (var, val)| {
                env.store(var, StateTerm::Term(val));
                env
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
                    let val = self.env.expand_value(*term);

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
                                    body.store(var, val);

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
                Term::Add(lhs, rhs) => vec![State {
                    env: self.env,
                    term: StateTerm::Term(Term::PM(PM::PMNat(PMNat {
                        var: lhs,
                        zero: Box::new(Term::Return(Box::new(Term::Var(rhs.clone())))),
                        succ: PMNatSucc {
                            var: "n".to_string(),
                            body: Box::new(Term::PM(PM::PMNat(PMNat {
                                var: rhs,
                                zero: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Var("n".to_string())))))),
                                succ: PMNatSucc {
                                    var: "m".to_string(),
                                    body: Box::new(Term::Bind {
                                        var: "0".to_string(),
                                        val: Box::new(Term::Add("n".to_string(), "m".to_string())),
                                        body: Box::new(Term::Return(Box::new(
                                            Term::Succ(Box::new(Term::Succ(Box::new(Term::Var("0".to_string())))))
                                        )))
                                    })
                                }
                            })))
                        }
                    }))),
                    stack: self.stack
                }],
                Term::Eq(lhs, rhs) => todo!(),
                Term::NEq(lhs, rhs) => todo!(),
                Term::Not(term) => todo!(),
                Term::If { cond, then, r#else } => todo!(),
                Term::App(lhs, rhs) => {
                    self.stack.push(StackTerm::Term(
                        self.env.lookup(&rhs).unwrap()
                    ));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(*lhs),
                        stack: self.stack
                    }]
                },
                Term::Force(term) => match self.env.lookup(&term).unwrap() {
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
                Term::Lambda { var, free_vars, body } => match self.stack.pop().unwrap() {
                    StackTerm::Term(term) => {
                        let mut closure = Closure::new(*body);
                        closure.store(var, term);

                        free_vars.into_iter()
                            .for_each(|var| {
                                let val = self.env.lookup(&var).unwrap();
                                closure.store(var, val);
                            });

                        vec![State {
                            env: self.env,
                            term: StateTerm::Closure(closure),
                            stack: self.stack
                        }]
                    },
                    _ => unreachable!()
                },
                Term::PM(pm) => match pm {
                    PM::PMNat(pm_nat) => match self.env.lookup(&pm_nat.var).unwrap() {
                        StateTerm::Term(term) => match term {
                            Term::Zero => vec![State {
                                env: self.env,
                                term: StateTerm::Term(*pm_nat.zero),
                                stack: self.stack
                            }],
                            Term::Succ(s) => {
                                self.stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                                self.env.store(pm_nat.succ.var, StateTerm::Term(*s));

                                vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(*pm_nat.succ.body),
                                    stack: self.stack
                                }]
                            },
                            Term::TypedVar(shape) => if shape.borrow().is_some() {
                                match shape.borrow().clone().unwrap() {
                                    Term::Zero => vec![State {
                                        env: self.env,
                                        term: StateTerm::Term(*pm_nat.zero),
                                        stack: self.stack
                                    }],
                                    Term::Succ(s) => {
                                        self.stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                                        self.env.store(pm_nat.succ.var, StateTerm::Term(*s));
    
                                        vec![State {
                                            env: self.env,
                                            term: StateTerm::Term(*pm_nat.succ.body),
                                            stack: self.stack
                                        }]
                                    },
                                    _ => unreachable!()
                                }
                            } else {
                                vec![
                                    {
                                        shape.replace(Some(Term::Zero));
    
                                        let mut new_locations = HashMap::new();
    
                                        State {
                                            env: self.env.clone_with_locations(&mut new_locations),
                                            term: StateTerm::Term(*pm_nat.zero),
                                            stack: self.stack.clone_with_locations(&mut new_locations)
                                        }
                                    },
                                    {
                                        let s = Term::TypedVar(Rc::new(RefCell::new(None)));
                                        shape.replace(Some(Term::Succ(Box::new(s.clone()))));
    
                                        self.stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                                        self.env.store(pm_nat.succ.var, StateTerm::Term(s));
    
                                        State {
                                            env: self.env,
                                            term: StateTerm::Term(*pm_nat.succ.body),
                                            stack: self.stack
                                        }
                                    }
                                ]
                            },
                            _ => unreachable!()
                        },
                        StateTerm::Closure(_) => unreachable!()
                    },
                    PM::PMList(_) => todo!()
                },
                Term::Choice(choices) => choices.into_iter()
                    .map(|choice| {
                        let mut new_locations = HashMap::new();

                        State {
                            env: self.env.clone_with_locations(&mut new_locations),
                            term: StateTerm::Term(choice),
                            stack: self.stack.clone_with_locations(&mut new_locations)
                        }
                    }).collect(),
                Term::Exists { var, body } => {
                    self.env.store(var, StateTerm::Term(Term::TypedVar(Rc::new(RefCell::new(None)))));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(*body),
                        stack: self.stack
                    }]
                },
                Term::Equate { lhs, rhs, body } => {
                    let lhs = match self.env.lookup(&lhs).unwrap() {
                        StateTerm::Term(term) => term,
                        StateTerm::Closure(_) => unreachable!()
                    };

                    let rhs = match self.env.lookup(&rhs).unwrap() {
                        StateTerm::Term(term) => term,
                        StateTerm::Closure(_) => unreachable!()
                    };

                    let flag = equate(lhs, rhs);

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(if flag {
                            *body
                        } else {
                            Term::Fail
                        }),
                        stack: self.stack
                    }]
                },
                Term::Fail => vec![State {
                    env: self.env,
                    term: StateTerm::Term(Term::Fail),
                    stack: self.stack
                }],
                _ => unreachable!(),
            },
            StateTerm::Closure(mut closure) => match closure.term.clone() {
                Term::Return(term) => {
                    let val = closure.expand_value(*term);

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
                                    body.store(var, val);

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
                    self.stack.push(StackTerm::Release("x".to_string()));
                    self.stack.push(StackTerm::Release("y".to_string()));

                    self.env.store("x".to_string(), closure.lookup(&lhs).unwrap());
                    self.env.store("y".to_string(), closure.lookup(&rhs).unwrap());

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(Term::Add("x".to_string(), "y".to_string())),
                        stack: self.stack
                    }]
                },
                Term::Eq(lhs, rhs) => todo!(),
                Term::NEq(lhs, rhs) => todo!(),
                Term::Not(term) => todo!(),
                Term::If { cond, then, r#else } => todo!(),
                Term::App(lhs, rhs) => {
                    self.stack.push(StackTerm::Term(
                        closure.lookup(&rhs).unwrap()
                    ));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term: *lhs, vars: closure.vars
                        }),
                        stack: self.stack
                    }]
                },
                Term::Force(term) => match closure.lookup(&term).unwrap() {
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
                Term::Lambda { var, free_vars,  body } => match self.stack.pop().unwrap() {
                    StackTerm::Term(term) => {
                        let mut state = Closure::new(*body);
                        state.store(var, term);

                        free_vars.into_iter()
                            .for_each(|var| {
                                let val = closure.lookup(&var).unwrap();
                                state.store(var, val);
                            });

                        vec![State {
                            env: self.env,
                            term: StateTerm::Closure(state),
                            stack: self.stack
                        }]
                    },
                    _ => unreachable!()
                },
                Term::Choice(choices) => choices.into_iter()
                    .map(|choice| {
                        let mut new_locations = HashMap::new();

                        State {
                            env: self.env.clone_with_locations(&mut new_locations),
                            term: StateTerm::Closure(Closure {
                                term: choice, vars: closure.clone_with_locations(&mut new_locations).vars
                            }),
                            stack: self.stack.clone_with_locations(&mut new_locations)
                        }
                    }).collect(),
                Term::Exists { var, body } => {
                    closure.store(
                        var,
                        StateTerm::Term(Term::TypedVar(Rc::new(RefCell::new(None))))
                    );
                    
                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term: *body, vars: closure.vars
                        }),
                        stack: self.stack
                    }]
                },
                Term::Equate { lhs, rhs, body } => {
                    let lhs = match closure.lookup(&lhs).unwrap() {
                        StateTerm::Term(term) => term,
                        StateTerm::Closure(_) => unreachable!()
                    };

                    let rhs = match closure.lookup(&rhs).unwrap() {
                        StateTerm::Term(term) => term,
                        StateTerm::Closure(_) => unreachable!()
                    };

                    let flag = equate(lhs, rhs);

                    vec![State {
                        env: self.env,
                        term: if flag {
                            StateTerm::Closure(Closure {
                                term: *body, vars: closure.vars
                            })
                        } else {
                            StateTerm::Term(Term::Fail)
                        },
                        stack: self.stack
                    }]
                },
                _ => unreachable!()
            }
        }
    }

    pub fn is_fail(&self) -> bool {
        match &self.term {
            StateTerm::Term(term) => match term {
                Term::Fail => true,
                _ => false
            },
            StateTerm::Closure(_) => false
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
