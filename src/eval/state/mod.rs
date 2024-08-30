use std::{cell::RefCell, collections::HashMap, rc::Rc};

use closure::{Closure, ClosureVars};
use env::{env::Env, env_value::{EnvValue, Shape, TypeVal}};
use env_lookup::EnvLookup;
use stack::{Stack, StackTerm};
use state_term::StateTerm;

use crate::cbpv::{PMSucc, Term};

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
                                    match &val {
                                        StateTerm::Term(term) => match term {
                                            Term::Var(v) => match self.env.lookup(v).unwrap() {
                                                EnvValue::Type(r#type) => self.env.bind(var.clone(), &r#type),
                                                EnvValue::Term(_) => unreachable!()
                                            },
                                            _ => self.env.store(var.clone(), val)
                                        },
                                        StateTerm::Closure(_) => self.env.store(var.clone(), val)
                                    }

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
                    match *lhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => {
                                self.stack.push(StackTerm::Release("x".to_string()));
                                self.env.store("x".to_string(), term);
                            },
                            EnvValue::Type(r#type) => self.env.bind("x".to_string(), &r#type)
                        },
                        _ => unreachable!()
                    }

                    match *rhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => {
                                self.stack.push(StackTerm::Release("y".to_string()));
                                self.env.store("y".to_string(), term);
                            },
                            EnvValue::Type(r#type) => self.env.bind("y".to_string(), &r#type)
                        }
                        _ => unreachable!()
                    }

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(Term::PM {
                            var: "x".to_string(),
                            zero: Box::new(Term::Return(Box::new(Term::Var("y".to_string())))),
                            succ: PMSucc {
                                var: "n".to_string(),
                                body: Box::new(Term::PM {
                                    var: "y".to_string(),
                                    zero: Box::new(Term::Return(Box::new(Term::Var("x".to_string())))),
                                    succ: PMSucc {
                                        var: "m".to_string(),
                                        body: Box::new(Term::Bind {
                                            var: "0".to_string(),
                                            val: Box::new(Term::Return(Box::new(Term::Var("n".to_string())))),
                                            body: Box::new(Term::Bind {
                                                var: "1".to_string(),
                                                val: Box::new(Term::Return(Box::new(Term::Var("m".to_string())))),
                                                body: Box::new(Term::Bind {
                                                    var: "2".to_string(),
                                                    val: Box::new(Term::Add(
                                                        Box::new(Term::Var("0".to_string())),
                                                        Box::new(Term::Var("1".to_string()))
                                                    )),
                                                    body: Box::new(Term::Return(Box::new(
                                                        Term::Succ(Box::new(Term::Succ(Box::new(Term::Var("2".to_string())))))
                                                    )))
                                                })
                                            })
                                        })
                                    }
                                })
                            }
                        }),
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
                Term::PM { var, zero, succ } => match self.env.lookup(&var).unwrap() {
                    EnvValue::Term(term) => match term {
                        StateTerm::Term(term) => match term {
                            Term::Zero => vec![State {
                                env: self.env,
                                term: StateTerm::Term(*zero),
                                stack: self.stack
                            }],
                            Term::Succ(term) => {
                                self.env.store(succ.var.clone(), StateTerm::Term(*term));
                                self.stack.push(StackTerm::Release(succ.var));

                                vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(*succ.body),
                                    stack: self.stack
                                }]
                            },
                            _ => unreachable!()
                        },
                        StateTerm::Closure(_) => unreachable!()
                    },
                    EnvValue::Type(r#type) => if r#type.borrow().val.is_some() {
                        match &r#type.borrow().val {
                            Some(shape) => match shape {
                                Shape::Zero => vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(*zero),
                                    stack: self.stack
                                }],
                                Shape::Succ(s) => {
                                    self.stack.push(StackTerm::Release(succ.var.clone()));
                                    self.env.bind(succ.var, &s);

                                    vec![State {
                                        env: self.env,
                                        term: StateTerm::Term(*succ.body),
                                        stack: self.stack
                                    }]
                                }
                            },
                            None => unreachable!()
                        }
                    } else {
                        vec![
                            {
                                r#type.borrow_mut().val = Some(Shape::Zero);
                                let env = self.env.clone();
                                
                                State {
                                    env,
                                    term: StateTerm::Term(*zero),
                                    stack: self.stack.clone()
                                }
                            },
                            {
                                let type_val = Rc::new(RefCell::new(TypeVal { val: None }));
                                r#type.borrow_mut().val = Some(Shape::Succ(Rc::clone(&type_val)));

                                self.stack.push(StackTerm::Release(succ.var.clone()));
                                self.env.bind(succ.var, &type_val);

                                State {
                                    env: self.env,
                                    term: StateTerm::Term(*succ.body),
                                    stack: self.stack
                                }
                            }
                        ]
                    }
                },
                Term::Choice(choices) => choices.into_iter()
                    .map(|choice| State {
                        env: self.env.clone(),
                        term: StateTerm::Term(choice),
                        stack: self.stack.clone()
                    }).collect(),
                Term::Exists { var, r#type: _, body } => {
                    self.env.bind(var.clone(), &Rc::new(RefCell::new(TypeVal { val: None })));
                    self.stack.push(StackTerm::Release(var));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Term(*body),
                        stack: self.stack
                    }]
                },
                Term::Equate { lhs, rhs, body } => {
                    let mut lhs = match *lhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => Term::Var(var)
                        },
                        _ => *lhs
                    };

                    let mut rhs = match *rhs {
                        Term::Var(var) => match self.env.lookup(&var).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(term) => term,
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(_) => Term::Var(var)
                        },
                        _ => *rhs
                    };

                    loop {
                        let mut flag = false;
                        
                        (lhs, rhs) = match lhs {
                            Term::Succ(new_lhs) => match rhs {
                                Term::Succ(new_rhs) => (*new_lhs, *new_rhs),
                                _ => {
                                    flag = true;
                                    (Term::Succ(new_lhs), rhs)
                                },
                            },
                            _ => {
                                flag = true;
                                (lhs, rhs)
                            },
                        };

                        if flag { break; }
                    }

                    match lhs {
                        Term::Var(var) => match rhs {
                            Term::Var(_) => vec![State {
                                env: self.env,
                                term: StateTerm::Term(*body),
                                stack: self.stack
                            }],
                            _ => {
                                self.env.release(&var);
                                self.env.store(var, StateTerm::Term(rhs));

                                vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(*body),
                                    stack: self.stack
                                }]
                            }
                        },
                        _ => match rhs {
                            Term::Var(var) => {
                                self.env.release(&var);
                                self.env.store(var, StateTerm::Term(lhs));

                                vec![State {
                                    env: self.env,
                                    term: StateTerm::Term(*body),
                                    stack: self.stack
                                }]
                            },
                            _ => vec![State {
                                env: self.env,
                                term: StateTerm::Term(if lhs == rhs {
                                    *body
                                } else {
                                    Term::Fail
                                }),
                                stack: self.stack
                            }]
                        }
                    }
                },
                Term::Fail => vec![State {
                    env: self.env,
                    term: StateTerm::Term(Term::Fail),
                    stack: self.stack
                }],
                _ => unreachable!(),
            },
            StateTerm::Closure(mut closure) => match closure.term {
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
                Term::Exists { var, r#type, body } => {
                    closure.vars.bind(var, r#type);
                    
                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term: *body, vars: closure.vars
                        }),
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

fn expand_value(term: Term, env: &impl EnvLookup) -> StateTerm {
    match term {
        Term::Zero => StateTerm::Term(term),
        Term::Bool(bool) => StateTerm::Term(Term::Bool(bool)),
        Term::Var(var) => match env.lookup(&var).unwrap() {
            EnvValue::Term(term) => term,
            EnvValue::Type(r#type) => match r#type.borrow().to_term() {
                Some(term) => StateTerm::Term(term),
                None => StateTerm::Term(Term::Var(var))
            }
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