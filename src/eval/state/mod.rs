use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use env::Env;
use equate::equate;
use stack::{Stack, StackTerm};
use state_term::{closure::Closure,state_term::{StateTerm, StateTermStore}};

use crate::{cbpv::{pm::{PMList, PMListCons, PMNat, PMNatSucc, PM}, term_ptr::TermPtr, Term}, parser::syntax::arg::Arg};

pub use state_term::locations_clone::LocationsClone;

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
                env.store(var, StateTerm::from_term(val));
                env
            });

        State {
            env,
            term: StateTerm::from_term(term),
            stack: Stack::new()
        }
    }

    pub fn step(mut self) -> Vec<State> {
        match self.term {
            StateTerm::Term(term) => match term.term() {
                Term::Return(term) => {
                    let val = self.env.expand_value(term.clone());

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
                                        StateTerm::Term(term_ptr) => StateTerm::from_term(Term::Return(term_ptr)),
                                        StateTerm::Closure(closure) => StateTerm::Closure(Closure {
                                            term_ptr: TermPtr::from_term(Term::Return(closure.term_ptr)), vars: closure.vars
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
                                StateTerm::Term(term_ptr) => StateTerm::from_term(Term::Return(term_ptr)),
                                StateTerm::Closure(closure) => StateTerm::Closure(Closure {
                                    term_ptr: TermPtr::from_term(Term::Return(closure.term_ptr)), vars: closure.vars
                                })
                            },
                            stack: self.stack
                        }]
                    }
                }
                Term::Bind { var, val, body } => {
                    self.stack.push(StackTerm::Cont(var.clone(), StateTerm::from_term_ptr(body.clone())));

                    vec![State {
                        env: self.env,
                        term: StateTerm::from_term_ptr(val.clone()),
                        stack: self.stack
                    }]
                },
                Term::Add(lhs, rhs) => vec![State {
                    env: self.env,
                    term: StateTerm::from_term(Term::PM(PM::PMNat(PMNat {
                        var: lhs.clone(),
                        zero: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var(rhs.clone())))),
                        succ: PMNatSucc {
                            var: "n".to_string(),
                            body: TermPtr::from_term(Term::PM(PM::PMNat(PMNat {
                                var: rhs.clone(),
                                zero: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Var("n".to_string())))))),
                                succ: PMNatSucc {
                                    var: "m".to_string(),
                                    body: TermPtr::from_term(Term::Bind {
                                        var: "0".to_string(),
                                        val: TermPtr::from_term(Term::Add("n".to_string(), "m".to_string())),
                                        body: TermPtr::from_term(Term::Return(TermPtr::from_term(
                                            Term::Succ(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Var("0".to_string())))))
                                        )))
                                    })
                                }
                            })))
                        }
                    }))),
                    stack: self.stack
                }],
                Term::Fold => vec![State {
                    env: self.env,
                    term: StateTerm::from_term(Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(Term::Lambda {
                        arg: Arg::Ident("f".to_string()),
                        free_vars: HashSet::new(),
                        body: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(Term::Lambda {
                            arg: Arg::Ident("z".to_string()),
                            free_vars: HashSet::from_iter(vec!["f".to_string()]),
                            body: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(Term::Lambda {
                                arg: Arg::Ident("xs".to_string()),
                                free_vars: HashSet::from_iter(vec!["f".to_string(), "z".to_string()]),
                                body: TermPtr::from_term(Term::PM(PM::PMList(PMList {
                                    var: "xs".to_string(),
                                    nil: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("z".to_string())))),
                                    cons: PMListCons {
                                        x: "y".to_string(),
                                        xs: "ys".to_string(),
                                        body: TermPtr::from_term(Term::Bind {
                                            var: "0".to_string(),
                                            val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("ys".to_string())))),
                                            body: TermPtr::from_term(Term::Bind {
                                                var: "1".to_string(),
                                                val: TermPtr::from_term(Term::Bind {
                                                    var: "0".to_string(),
                                                    val: TermPtr::from_term(Term::Bind {
                                                        var: "0".to_string(),
                                                        val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("y".to_string())))),
                                                        body: TermPtr::from_term(Term::Bind {
                                                            var: "1".to_string(),
                                                            val: TermPtr::from_term(Term::Bind {
                                                                var: "0".to_string(),
                                                                val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("z".to_string())))),
                                                                body: TermPtr::from_term(Term::Bind {
                                                                    var: "1".to_string(),
                                                                    val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("f".to_string())))),
                                                                    body: TermPtr::from_term(Term::App(
                                                                        TermPtr::from_term(Term::Force("1".to_string())),
                                                                        "0".to_string()
                                                                    ))
                                                                })
                                                            }),
                                                            body: TermPtr::from_term(Term::App(
                                                                TermPtr::from_term(Term::Force("1".to_string())),
                                                                "0".to_string()
                                                            ))
                                                        })
                                                    }),
                                                    body: TermPtr::from_term(Term::Bind {
                                                        var: "1".to_string(),
                                                        val: TermPtr::from_term(Term::Bind {
                                                            var: "0".to_string(),
                                                            val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("f".to_string())))),
                                                            body: TermPtr::from_term(Term::Bind {
                                                                var: "1".to_string(),
                                                                val: TermPtr::from_term(Term::Fold),
                                                                body: TermPtr::from_term(Term::App(
                                                                    TermPtr::from_term(Term::Force("1".to_string())),
                                                                    "0".to_string()
                                                                ))
                                                            })
                                                        }),
                                                        body: TermPtr::from_term(Term::App(
                                                            TermPtr::from_term(Term::Force("1".to_string())),
                                                            "0".to_string()
                                                        ))
                                                    })
                                                }),
                                                body: TermPtr::from_term(Term::App(
                                                    TermPtr::from_term(Term::Force("1".to_string())),
                                                    "0".to_string()
                                                ))
                                            })
                                        })
                                    }
                                })))
                            })))))
                        })))))
                    }))))),
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
                        term: StateTerm::from_term_ptr(lhs.clone()),
                        stack: self.stack
                    }]
                },
                Term::Force(term) => match self.env.lookup(&term).unwrap() {
                    StateTerm::Term(term_ptr) => match term_ptr.term() {
                        Term::Thunk(term_ptr) => vec![State {
                            env: self.env,
                            term: StateTerm::from_term_ptr(term_ptr.clone()),
                            stack: self.stack
                        }],
                        _ => unreachable!()
                    },
                    StateTerm::Closure(closure) => match closure.term_ptr.term() {
                        Term::Thunk(term_ptr) => vec![State {
                            env: self.env,
                            term: StateTerm::Closure(Closure {
                                term_ptr: term_ptr.clone(), vars: closure.vars
                            }),
                            stack: self.stack
                        }],
                        _ => unreachable!()
                    }
                },
                Term::Lambda { arg, free_vars, body } => match self.stack.pop().unwrap() {
                    StackTerm::Term(term) => {
                        let mut closure = Closure::from_term_ptr(body.clone());
                        match arg {
                            Arg::Ident(var) => { closure.store(var.clone(), term); },
                            Arg::Pair(lhs, rhs) => match term {
                                StateTerm::Term(term) => match term.term() {
                                    Term::Pair(lhs_val, rhs_val) => {
                                        closure.store(lhs.clone(), StateTerm::from_term_ptr(lhs_val.clone()));
                                        closure.store(rhs.clone(), StateTerm::from_term_ptr(rhs_val.clone()));
                                    },
                                    _ => unreachable!()
                                },
                                StateTerm::Closure(_) => unreachable!()
                            }
                        }

                        free_vars.into_iter()
                            .for_each(|var| {
                                let val = self.env.lookup(&var).unwrap();
                                closure.store(var.clone(), val);
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
                        StateTerm::Term(term_ptr) => match term_ptr.term() {
                            Term::Zero => vec![State {
                                env: self.env,
                                term: StateTerm::from_term_ptr(pm_nat.zero.clone()),
                                stack: self.stack
                            }],
                            Term::Succ(s) => {
                                self.stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                                self.env.store(pm_nat.succ.var.clone(), StateTerm::from_term_ptr(s.clone()));

                                vec![State {
                                    env: self.env,
                                    term: StateTerm::from_term_ptr(pm_nat.succ.body.clone()),
                                    stack: self.stack
                                }]
                            },
                            Term::TypedVar(shape) => if shape.borrow().is_some() {
                                match shape.borrow().as_ref().unwrap().term() {
                                    Term::Zero => vec![State {
                                        env: self.env,
                                        term: StateTerm::from_term_ptr(pm_nat.zero.clone()),
                                        stack: self.stack
                                    }],
                                    Term::Succ(s) => {
                                        self.stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                                        self.env.store(pm_nat.succ.var.clone(), StateTerm::from_term_ptr(s.clone()));
    
                                        vec![State {
                                            env: self.env,
                                            term: StateTerm::from_term_ptr(pm_nat.succ.body.clone()),
                                            stack: self.stack
                                        }]
                                    },
                                    _ => unreachable!()
                                }
                            } else {
                                vec![
                                    {    
                                        let mut new_locations = HashMap::new();

                                        let env = self.env.clone_with_locations(&mut new_locations);
                                        let stack = self.stack.clone_with_locations(&mut new_locations);

                                        let shape = match env.lookup(&pm_nat.var).unwrap() {
                                            StateTerm::Term(term_ptr) => match term_ptr.term() {
                                                Term::TypedVar(shape) => Rc::clone(shape),
                                                _ => unreachable!()
                                            },
                                            StateTerm::Closure(_) => unreachable!()
                                        };

                                        shape.replace(Some(TermPtr::from_term(Term::Zero)));
    
                                        State {
                                            env,
                                            term: StateTerm::from_term_ptr(pm_nat.zero.clone()),
                                            stack
                                        }
                                    },
                                    {
                                        let s = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                                        shape.replace(Some(TermPtr::from_term(Term::Succ(s.clone()))));
    
                                        self.stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                                        self.env.store(pm_nat.succ.var.clone(), StateTerm::from_term_ptr(s));
    
                                        State {
                                            env: self.env,
                                            term: StateTerm::from_term_ptr(pm_nat.succ.body.clone()),
                                            stack: self.stack
                                        }
                                    }
                                ]
                            },
                            _ => unreachable!()
                        },
                        StateTerm::Closure(_) => unreachable!()
                    },
                    PM::PMList(_) => unreachable!()
                },
                Term::Choice(choices) => choices.into_iter()
                    .map(|choice| {
                        let mut new_locations = HashMap::new();

                        State {
                            env: self.env.clone_with_locations(&mut new_locations),
                            term: StateTerm::from_term_ptr(choice.clone()),
                            stack: self.stack.clone_with_locations(&mut new_locations)
                        }
                    }).collect(),
                Term::Exists { var, body } => {
                    self.env.store(var.clone(), StateTerm::from_term(Term::TypedVar(Rc::new(RefCell::new(None)))));

                    vec![State {
                        env: self.env,
                        term: StateTerm::from_term_ptr(body.clone()),
                        stack: self.stack
                    }]
                },
                Term::Equate { lhs, rhs, body } => {
                    let lhs = match self.env.lookup(&lhs).unwrap() {
                        StateTerm::Term(term_ptr) => term_ptr,
                        StateTerm::Closure(_) => unreachable!()
                    };

                    let rhs = match self.env.lookup(&rhs).unwrap() {
                        StateTerm::Term(term_ptr) => term_ptr,
                        StateTerm::Closure(_) => unreachable!()
                    };

                    let flag = equate(lhs, rhs);

                    vec![State {
                        env: self.env,
                        term: StateTerm::from_term_ptr(if flag {
                            body.clone()
                        } else {
                            TermPtr::from_term(Term::Fail)
                        }),
                        stack: self.stack
                    }]
                },
                Term::Fail => vec![State {
                    env: self.env,
                    term: StateTerm::from_term(Term::Fail),
                    stack: self.stack
                }],
                _ => unreachable!(),
            },
            StateTerm::Closure(mut closure) => match closure.term_ptr.clone().term() {
                Term::Return(term_ptr) => {
                    let val = closure.expand_value(term_ptr.clone());

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
                                        StateTerm::Term(term_ptr) => StateTerm::from_term(Term::Return(term_ptr)),
                                        StateTerm::Closure(closure) => StateTerm::Closure(Closure {
                                            term_ptr: TermPtr::from_term(Term::Return(closure.term_ptr)), vars: closure.vars
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
                    self.stack.push(StackTerm::Cont(var.clone(), StateTerm::Closure(Closure {
                        term_ptr: body.clone(), vars: closure.vars.clone()
                    })));

                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term_ptr: val.clone(), vars: closure.vars
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
                        term: StateTerm::from_term(Term::Add("x".to_string(), "y".to_string())),
                        stack: self.stack
                    }]
                },
                Term::Fold => vec![State {
                    env: self.env,
                    term: StateTerm::from_term(Term::Fold),
                    stack: self.stack
                }],
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
                            term_ptr: lhs.clone(), vars: closure.vars
                        }),
                        stack: self.stack
                    }]
                },
                Term::Force(term) => match closure.lookup(&term).unwrap() {
                    StateTerm::Term(term_ptr) => match term_ptr.term() {
                        Term::Thunk(term_ptr) => vec![State {
                            env: self.env,
                            term: StateTerm::from_term_ptr(term_ptr.clone()),
                            stack: self.stack
                        }],
                        _ => unreachable!()
                    },
                    StateTerm::Closure(closure) => match closure.term_ptr.term() {
                        Term::Thunk(term_ptr) => vec![State {
                            env: self.env,
                            term: StateTerm::Closure(Closure {
                                term_ptr: term_ptr.clone(), vars: closure.vars
                            }),
                            stack: self.stack
                        }],
                        _ => unreachable!()
                    }
                },
                Term::Lambda { arg, free_vars,  body } => match self.stack.pop().unwrap() {
                    StackTerm::Term(term) => {
                        let mut state = Closure::from_term_ptr(body.clone());
                        match arg {
                            Arg::Ident(var) => { state.store(var.clone(), term); },
                            Arg::Pair(_, _) => todo!()
                        }

                        free_vars.into_iter()
                            .for_each(|var| {
                                let val = closure.lookup(var).unwrap();
                                state.store(var.clone(), val);
                            });

                        vec![State {
                            env: self.env,
                            term: StateTerm::Closure(state),
                            stack: self.stack
                        }]
                    },
                    _ => unreachable!()
                },
                Term::PM(pm) => match pm {
                    PM::PMList(pm_list) => match closure.lookup(&pm_list.var).unwrap() {
                        StateTerm::Term(term_ptr) => match term_ptr.term() {
                            Term::Nil => vec![State {
                                env: self.env,
                                term: StateTerm::Closure(Closure {
                                    term_ptr: pm_list.nil.clone(), vars: closure.vars
                                }),
                                stack: self.stack
                            }],
                            Term::Cons(x, xs) => {
                                closure.store(pm_list.cons.x.clone(), StateTerm::from_term_ptr(x.clone()));
                                closure.store(pm_list.cons.xs.clone(), StateTerm::from_term_ptr(xs.clone()));

                                vec![State {
                                    env: self.env,
                                    term: StateTerm::Closure(Closure {
                                        term_ptr: pm_list.cons.body.clone(), vars: closure.vars
                                    }),
                                    stack: self.stack
                                }]
                            },
                            Term::TypedVar(shape) => if shape.borrow().is_some() {
                                match shape.borrow().as_ref().unwrap().term() {
                                    Term::Nil => vec![State {
                                        env: self.env,
                                        term: StateTerm::Closure(Closure {
                                            term_ptr: pm_list.nil.clone(), vars: closure.vars
                                        }),
                                        stack: self.stack
                                    }],
                                    Term::Cons(x, xs) => {
                                        closure.store(pm_list.cons.x.clone(), StateTerm::from_term_ptr(x.clone()));
                                        closure.store(pm_list.cons.xs.clone(), StateTerm::from_term_ptr(xs.clone()));

                                        vec![State {
                                            env: self.env,
                                            term: StateTerm::Closure(Closure {
                                                term_ptr: pm_list.cons.body.clone(), vars: closure.vars
                                            }),
                                            stack: self.stack
                                        }]
                                    },
                                    _ => unreachable!()
                                }
                            } else {
                                vec![
                                    {
                                        let mut new_locations = HashMap::new();

                                        let env = self.env.clone_with_locations(&mut new_locations);
                                        let closure = closure.clone_with_locations(&mut new_locations);
                                        let stack = self.stack.clone_with_locations(&mut new_locations);

                                        let shape = match closure.lookup(&pm_list.var).unwrap() {
                                            StateTerm::Term(term_ptr) => match term_ptr.term() {
                                                Term::TypedVar(shape) => Rc::clone(shape),
                                                _ => unreachable!()
                                            },
                                            StateTerm::Closure(_) => unreachable!()
                                        };

                                        shape.replace(Some(TermPtr::from_term(Term::Nil)));

                                        State {
                                            env,
                                            term: StateTerm::Closure(Closure {
                                                term_ptr: pm_list.nil.clone(), vars: closure.vars
                                            }),
                                            stack
                                        }
                                    },
                                    {
                                        let x = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                                        let xs = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));

                                        shape.replace(Some(TermPtr::from_term(Term::Cons(x.clone(), xs.clone()))));

                                        closure.store(pm_list.cons.x.clone(), StateTerm::from_term_ptr(x));
                                        closure.store(pm_list.cons.xs.clone(), StateTerm::from_term_ptr(xs));

                                        State {
                                            env: self.env,
                                            term: StateTerm::Closure(Closure {
                                                term_ptr: pm_list.cons.body.clone(), vars: closure.vars
                                            }),
                                            stack: self.stack
                                        }
                                    }
                                ]
                            },
                            _ => unreachable!()
                        },
                        StateTerm::Closure(_) => unreachable!()
                    },
                    PM::PMNat(_) => unreachable!()
                },
                Term::Choice(choices) => choices.into_iter()
                    .map(|choice| {
                        let mut new_locations = HashMap::new();

                        State {
                            env: self.env.clone_with_locations(&mut new_locations),
                            term: StateTerm::Closure(Closure {
                                term_ptr: choice.clone(), vars: closure.clone_with_locations(&mut new_locations).vars
                            }),
                            stack: self.stack.clone_with_locations(&mut new_locations)
                        }
                    }).collect(),
                Term::Exists { var, body } => {
                    closure.store(
                        var.clone(),
                        StateTerm::from_term(Term::TypedVar(Rc::new(RefCell::new(None))))
                    );
                    
                    vec![State {
                        env: self.env,
                        term: StateTerm::Closure(Closure {
                            term_ptr: body.clone(), vars: closure.vars
                        }),
                        stack: self.stack
                    }]
                },
                Term::Equate { lhs, rhs, body } => {
                    let lhs = match closure.lookup(&lhs).unwrap() {
                        StateTerm::Term(term_ptr) => term_ptr,
                        StateTerm::Closure(_) => unreachable!()
                    };

                    let rhs = match closure.lookup(&rhs).unwrap() {
                        StateTerm::Term(term_ptr) => term_ptr,
                        StateTerm::Closure(_) => unreachable!()
                    };

                    let flag = equate(lhs, rhs);

                    vec![State {
                        env: self.env,
                        term: if flag {
                            StateTerm::Closure(Closure {
                                term_ptr: body.clone(), vars: closure.vars
                            })
                        } else {
                            StateTerm::from_term(Term::Fail)
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
            StateTerm::Term(term_ptr) => match term_ptr.term() {
                Term::Fail => true,
                _ => false
            },
            StateTerm::Closure(_) => false
        }
    }

    pub fn is_value(&self) -> bool {
        if self.stack.is_empty() {
            match &self.term {
                StateTerm::Term(term_ptr) => match term_ptr.term() {
                    Term::Return(term_ptr) => match term_ptr.term() {
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

    pub fn term(self) -> TermPtr {
        match self.term {
            StateTerm::Term(term_ptr) => term_ptr.clone(),
            StateTerm::Closure(_) => unreachable!()
        }
    }
}
