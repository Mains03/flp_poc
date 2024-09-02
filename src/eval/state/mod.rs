use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use closure::{Closure, ClosureVars};
use env_lookup::EnvLookup;
use frame::{env::env_value::{EnvValue, Shape, TypeVal}, stack::StackTerm, Frame};
use state_term::StateTerm;

use crate::cbpv::{PMSucc, Term};

mod state_term;
mod env_lookup;
mod closure;
mod frame;

#[derive(Debug)]
pub struct State {
    frame: Frame,
    term: StateTerm
}

impl State {
    pub fn new(mut cbpv: HashMap<String, Term>) -> Self {
        let term = cbpv.remove("main").unwrap();

        let mut frame = Frame::new();
        cbpv.into_iter()
            .for_each(|(var, val)| {
                frame.env().store(var, StateTerm::Term(val))
            });

        State {
            frame,
            term: StateTerm::Term(term),
        }
    }

    pub fn step(mut self) -> Vec<State> {
        match self.term {
            StateTerm::Term(term) => match term {
                Term::Return(term) => {
                    let val = expand_value(*term, self.frame.env());

                    match self.frame.stack().pop() {
                        Some(s) => match s {
                            StackTerm::Cont(var, body) => match body {
                                StateTerm::Term(_) => {
                                    match &val {
                                        StateTerm::Term(term) => match term {
                                            Term::Var(v) => match self.frame.env().lookup(v).unwrap() {
                                                EnvValue::Type(r#type) => self.frame.env().bind(var.clone(), &r#type),
                                                EnvValue::Term(_) => unreachable!()
                                            },
                                            _ => self.frame.env().store(var.clone(), val)
                                        },
                                        StateTerm::Closure(_) => self.frame.env().store(var.clone(), val)
                                    }

                                    self.frame.stack().push(StackTerm::Release(var));

                                    vec![State {
                                        frame: self.frame,
                                        term: body
                                    }]
                                },
                                StateTerm::Closure(mut body) => {
                                    body.vars.store(var, val);

                                    vec![State {
                                        frame: self.frame,
                                        term: StateTerm::Closure(body),
                                    }]
                                }
                            },
                            StackTerm::Release(var) => {
                                self.frame.env().release(&var);

                                vec![State {
                                    frame: self.frame,
                                    term: match val {
                                        StateTerm::Term(term) => StateTerm::Term(Term::Return(Box::new(term))),
                                        StateTerm::Closure(closure) => StateTerm::Closure(Closure {
                                            term: Term::Return(Box::new(closure.term)), vars: closure.vars
                                        })
                                    },
                                }]
                            },
                            StackTerm::Term(_) => unreachable!()
                        },
                        None => vec![State {
                            frame: self.frame,
                            term: match val {
                                StateTerm::Term(term) => StateTerm::Term(Term::Return(Box::new(term))),
                                StateTerm::Closure(closure) => StateTerm::Closure(Closure {
                                    term: Term::Return(Box::new(closure.term)), vars: closure.vars
                                })
                            },
                        }]
                    }
                }
                Term::Bind { var, val, body } => {
                    self.frame.stack().push(StackTerm::Cont(var, StateTerm::Term(*body)));

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(*val),
                    }]
                },
                Term::Add(lhs, rhs) => {
                    self.frame.stack().push(StackTerm::Release("x".to_string()));
                    self.frame.stack().push(StackTerm::Release("y".to_string()));

                    match self.frame.env().lookup(&lhs).unwrap() {
                        EnvValue::Term(term) => {
                            self.frame.env().store("x".to_string(), term);
                        },
                        EnvValue::Type(r#type) => self.frame.env().bind("x".to_string(), &r#type)
                    }

                    match self.frame.env().lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => {
                            self.frame.env().store("y".to_string(), term);
                        },
                        EnvValue::Type(r#type) => self.frame.env().bind("y".to_string(), &r#type)
                    }

                    vec![State {
                        frame: self.frame,
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
                                                    val: Box::new(Term::Add("0".to_string(), "1".to_string())),
                                                    body: Box::new(Term::Return(Box::new(
                                                        Term::Succ(Box::new(Term::Succ(Box::new(Term::Var("2".to_string())))))
                                                    )))
                                                })
                                            })
                                        })
                                    }
                                })
                            }
                        })
                    }]
                },
                Term::Eq(lhs, rhs) => {
                    let lhs = match self.frame.env().lookup(&lhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => term,
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => todo!()
                    };

                    let rhs = match self.frame.env().lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => term,
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => todo!()
                    };

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(Term::Return(Box::new(Term::Bool(lhs == rhs))))
                    }]
                },
                Term::NEq(lhs, rhs) => {
                    let lhs = match self.frame.env().lookup(&lhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => term,
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => todo!()
                    };

                    let rhs = match self.frame.env().lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => term,
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => todo!()
                    };

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(Term::Return(Box::new(Term::Bool(lhs != rhs))))
                    }]
                },
                Term::Not(term) => match self.frame.env().lookup(&term).unwrap() {
                    EnvValue::Term(term) => match term {
                        StateTerm::Term(term) => match term {
                            Term::Bool(bool) => vec![State {
                                frame: self.frame,
                                term: StateTerm::Term(Term::Return(Box::new(Term::Bool(!bool))))
                            }],
                            _ => unreachable!()
                        },
                        StateTerm::Closure(_) => unreachable!()
                    },
                    EnvValue::Type(_) => unreachable!()
                },
                Term::If { cond, then, r#else } => {
                    let term = match self.frame.env().lookup(&cond).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => match term {
                                Term::Bool(bool) => if bool { *then } else { *r#else },
                                _ => unreachable!()
                            },
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => unreachable!()
                    };

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(term)
                    }]
                },
                Term::App(lhs, rhs) => {
                    let rhs = match self.frame.env().lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => term,
                        EnvValue::Type(_) => todo!()
                    };

                    self.frame.stack().push(StackTerm::Term(rhs));

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(*lhs)
                    }]
                },
                Term::Force(term) => match self.frame.env().lookup(&term).unwrap() {
                    EnvValue::Term(term) => match term {
                        StateTerm::Term(term) => match term {
                            Term::Thunk(term) => vec![State {
                                frame: self.frame,
                                term: StateTerm::Term(*term),
                            }],
                            _ => unreachable!()
                        },
                        StateTerm::Closure(closure) => match closure.term {
                            Term::Thunk(term) => vec![State {
                                frame: self.frame,
                                term: StateTerm::Closure(Closure {
                                    term: *term, vars: closure.vars
                                })
                            }],
                            _ => unreachable!()
                        }
                    },
                    EnvValue::Type(_) => unreachable!()
                },
                Term::Lambda { var, free_vars, body } => match self.frame.stack().pop().unwrap() {
                    StackTerm::Term(term) => {
                        let mut vars = ClosureVars::new();
                        vars.store(var, term);

                        free_vars.into_iter()
                            .for_each(|var| {
                                match self.frame.env().lookup(&var).unwrap() {
                                    EnvValue::Term(term) => vars.store(var, term),
                                    EnvValue::Type(_) => todo!()
                                }
                            });

                        vec![State {
                            frame: self.frame,
                            term: StateTerm::Closure(Closure {
                                term: *body, vars
                            })
                        }]
                    },
                    _ => unreachable!()
                },
                Term::PM { var, zero, succ } => match self.frame.env().lookup(&var).unwrap() {
                    EnvValue::Term(term) => match term {
                        StateTerm::Term(term) => match term {
                            Term::Zero => vec![State {
                                frame: self.frame,
                                term: StateTerm::Term(*zero)
                            }],
                            Term::Succ(term) => {
                                self.frame.env().store(succ.var.clone(), StateTerm::Term(*term));
                                self.frame.stack().push(StackTerm::Release(succ.var));

                                vec![State {
                                    frame: self.frame,
                                    term: StateTerm::Term(*succ.body)
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
                                    frame: self.frame,
                                    term: StateTerm::Term(*zero)
                                }],
                                Shape::Succ(s) => {
                                    self.frame.stack().push(StackTerm::Release(succ.var.clone()));
                                    self.frame.env().bind(succ.var, &s);

                                    vec![State {
                                        frame: self.frame,
                                        term: StateTerm::Term(*succ.body)
                                    }]
                                }
                            },
                            None => unreachable!()
                        }
                    } else {
                        vec![
                            {
                                r#type.borrow_mut().val = Some(Shape::Zero);
                                let frame = self.frame.clone();
                                
                                State {
                                    frame,
                                    term: StateTerm::Term(*zero)
                                }
                            },
                            {
                                let type_val = Rc::new(RefCell::new(TypeVal { val: None }));
                                r#type.borrow_mut().val = Some(Shape::Succ(Rc::clone(&type_val)));

                                self.frame.stack().push(StackTerm::Release(succ.var.clone()));
                                self.frame.env().bind(succ.var, &type_val);

                                State {
                                    frame: self.frame,
                                    term: StateTerm::Term(*succ.body)
                                }
                            }
                        ]
                    }
                },
                Term::Choice(choices) => choices.into_iter()
                    .map(|choice| State {
                        frame: self.frame.clone(),
                        term: StateTerm::Term(choice)
                    }).collect(),
                Term::Exists { var, r#type: _, body } => {
                    self.frame.env().bind(var.clone(), &Rc::new(RefCell::new(TypeVal { val: None })));
                    self.frame.stack().push(StackTerm::Release(var));

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(*body)
                    }]
                },
                Term::Equate { lhs, rhs, body } => match self.frame.env().lookup(&lhs).unwrap() {
                    EnvValue::Term(term) => match term {
                        StateTerm::Term(lhs_term) => match self.frame.env().lookup(&rhs).unwrap() {
                            EnvValue::Term(term) => match term {
                                StateTerm::Term(rhs_term) => vec![State {
                                    frame: self.frame,
                                    term: StateTerm::Term(if lhs_term == rhs_term {
                                        *body
                                    } else {
                                        Term::Fail
                                    })
                                }],
                                StateTerm::Closure(_) => unreachable!()
                            },
                            EnvValue::Type(r#type) => {
                                r#type.borrow_mut().set_shape(&lhs_term);

                                vec![State {
                                    frame: self.frame,
                                    term: StateTerm::Term(*body)
                                }]
                            }
                        },
                        StateTerm::Closure(_) => unreachable!()
                    },
                    EnvValue::Type(r#type) => match self.frame.env().lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => {
                                r#type.borrow_mut().set_shape(&term);

                                vec![State {
                                    frame: self.frame,
                                    term: StateTerm::Term(*body)
                                }]
                            },
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => vec![State {
                            frame: self.frame,
                            term: StateTerm::Term(*body)
                        }]
                    }
                },
                Term::Fail => vec![State {
                    frame: self.frame,
                    term: StateTerm::Term(Term::Fail)
                }],
                _ => unreachable!(),
            },
            StateTerm::Closure(mut closure) => match closure.term {
                Term::Return(term) => {
                    let val = expand_closure_value(*term, &closure.vars);

                    match self.frame.stack().pop() {
                        Some(s) => match s {
                            StackTerm::Cont(var, body) => match body {
                                StateTerm::Term(_) => {
                                    match &val {
                                        StateTerm::Term(term) => match term {
                                            Term::Var(v) => match closure.vars.lookup(&v).unwrap() {
                                                EnvValue::Type(r#type) => self.frame.env().bind(var.clone(), &r#type),
                                                EnvValue::Term(_) => unreachable!()
                                            },
                                            _ => self.frame.env().store(var.clone(), val)
                                        },
                                        StateTerm::Closure(_) => self.frame.env().store(var.clone(), val)
                                    }

                                    self.frame.stack().push(StackTerm::Release(var));

                                    vec![State {
                                        frame: self.frame,
                                        term: body
                                    }]
                                },
                                StateTerm::Closure(mut body) => {
                                    match &val {
                                        StateTerm::Term(term) => match term {
                                            Term::Var(v) => match closure.vars.lookup(&v).unwrap() {
                                                EnvValue::Type(r#type) => body.vars.bind(var.clone(), &r#type),
                                                EnvValue::Term(_) => unreachable!()
                                            },
                                            _ => body.vars.store(var.clone(), val)
                                        },
                                        StateTerm::Closure(_) => body.vars.store(var.clone(), val)
                                    }

                                    vec![State {
                                        frame: self.frame,
                                        term: StateTerm::Closure(body)
                                    }]
                                }
                            },
                            StackTerm::Release(var) => {
                                self.frame.env().release(&var);

                                vec![State {
                                    frame: self.frame,
                                    term: match val {
                                        StateTerm::Term(term) => StateTerm::Term(Term::Return(Box::new(term))),
                                        StateTerm::Closure(val) => StateTerm::Closure(Closure {
                                            term: Term::Return(Box::new(val.term)), vars: val.vars
                                        })
                                    }
                                }]
                            },
                            StackTerm::Term(_) => unreachable!()
                        },
                        None => unreachable!()
                    }
                },
                Term::Bind { var, val, body } => {
                    self.frame.stack().push(StackTerm::Cont(var, StateTerm::Closure(Closure {
                        term: *body, vars: closure.vars.clone()
                    })));

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Closure(Closure {
                            term: *val, vars: closure.vars
                        })
                    }]
                },
                Term::Add(lhs, rhs) => {
                    self.frame.stack().push(StackTerm::Release("x".to_string()));
                    self.frame.stack().push(StackTerm::Release("y".to_string()));

                    match closure.vars.lookup(&lhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => {
                                self.frame.env().store("x".to_string(), StateTerm::Term(term));
                            },
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(r#type) => {
                            self.frame.env().bind("x".to_string(), &r#type);
                        }
                    }

                    match closure.vars.lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => {
                                self.frame.env().store("y".to_string(), StateTerm::Term(term));
                            },
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(r#type) => {
                            self.frame.env().bind("y".to_string(), &r#type);
                        }
                    }

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(Term::Add("x".to_string(), "y".to_string()))
                    }]
                },
                Term::Eq(lhs, rhs) => {
                    let lhs = match closure.vars.lookup(&lhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => term,
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => todo!()
                    };

                    let rhs = match closure.vars.lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => term,
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => todo!()
                    };

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(Term::Return(Box::new(Term::Bool(lhs == rhs))))
                    }]
                },
                Term::NEq(lhs, rhs) => {
                    let lhs = match closure.vars.lookup(&lhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => term,
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => todo!()
                    };

                    let rhs = match closure.vars.lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => term,
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => todo!()
                    };

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(Term::Return(Box::new(Term::Bool(lhs != rhs))))
                    }]
                },
                Term::Not(term) => match closure.vars.lookup(&term).unwrap() {
                    EnvValue::Term(term) => match term {
                        StateTerm::Term(term) => match term {
                            Term::Bool(bool) => vec![State {
                                frame: self.frame,
                                term: StateTerm::Term(Term::Return(Box::new(Term::Bool(!bool))))
                            }],
                            _ => unreachable!()
                        },
                        StateTerm::Closure(_) => unreachable!()
                    },
                    EnvValue::Type(_) => unreachable!()
                },
                Term::If { cond, then, r#else } => {
                    let term = match closure.vars.lookup(&cond).unwrap() {
                        EnvValue::Term(term) => match term {
                            StateTerm::Term(term) => match term {
                                Term::Bool(bool) => if bool { *then } else { *r#else },
                                _ => unreachable!()
                            },
                            StateTerm::Closure(_) => unreachable!()
                        },
                        EnvValue::Type(_) => unreachable!()
                    };

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Closure(Closure {
                            term, vars: closure.vars
                        })
                    }]
                },
                Term::App(lhs, rhs) => {
                    let rhs = match closure.vars.lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => term,
                        EnvValue::Type(_) => todo!()
                    };

                    self.frame.stack().push(StackTerm::Term(rhs));

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Closure(Closure {
                            term: *lhs, vars: closure.vars
                        })
                    }]
                },
                Term::Force(term) => match closure.vars.lookup(&term).unwrap() {
                    EnvValue::Term(term) => match term {
                        StateTerm::Term(term) => match term {
                            Term::Thunk(term) => vec![State {
                                frame: self.frame,
                                term: StateTerm::Term(*term)
                            }],
                            _ => unreachable!()
                        },
                        StateTerm::Closure(closure) => match closure.term {
                            Term::Thunk(term) => vec![State {
                                frame: self.frame,
                                term: StateTerm::Closure(Closure {
                                    term: *term, vars: closure.vars
                                })
                            }],
                            _ => unreachable!()
                        }
                    },
                    EnvValue::Type(_) => unreachable!()
                },
                Term::Lambda { var, free_vars,  body } => match self.frame.stack().pop().unwrap() {
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
                            frame: self.frame,
                            term: StateTerm::Closure(Closure {
                                term: *body, vars
                            })
                        }]
                    },
                    _ => unreachable!()
                },
                Term::Exists { var, r#type: _, body } => {
                    closure.vars.bind(var, &Rc::new(RefCell::new(TypeVal {
                        val: None
                    })));
                    
                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Closure(Closure {
                            term: *body, vars: closure.vars
                        })
                    }]
                },
                Term::Equate { lhs, rhs, body } => {
                    self.frame.stack().push(StackTerm::Release("0".to_string()));
                    self.frame.stack().push(StackTerm::Release("1".to_string()));

                    match closure.vars.lookup(&lhs).unwrap() {
                        EnvValue::Term(term) => self.frame.env().store("0".to_string(), term),
                        EnvValue::Type(r#type) => self.frame.env().bind("0".to_string(), &r#type)
                    }

                    match closure.vars.lookup(&rhs).unwrap() {
                        EnvValue::Term(term) => self.frame.env().store("1".to_string(), term),
                        EnvValue::Type(r#type) => self.frame.env().bind("1".to_string(), &r#type)
                    }

                    self.frame.stack().push(StackTerm::Term(StateTerm::Closure(Closure {
                        term: *body, vars: closure.vars
                    })));

                    vec![State {
                        frame: self.frame,
                        term: StateTerm::Term(Term::Equate {
                            lhs: "0".to_string(),
                            rhs: "1".to_string(),
                            body: Box::new(Term::Lambda {
                                var: "x".to_string(),
                                free_vars: HashSet::new(),
                                body: Box::new(Term::Return(Box::new(Term::Var("x".to_string()))))
                            })
                        })
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
        if self.frame.stack_ref().is_empty() {
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

fn expand_closure_value(term: Term, vars: &ClosureVars) -> StateTerm {
    match term {
        Term::Thunk(_) => StateTerm::Closure(Closure {
            term, vars: vars.clone()
        }),
        _ => expand_value(term, vars)
    }
}
