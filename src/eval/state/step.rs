use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{cbpv::{pm::{PMNat, PMNatSucc, PM}, term_ptr::TermPtr, Term}, eval::{state::equate::equate, LocationsClone}};

use super::{env::Env, stack::{Stack, StackTerm}, state_term::{closure::{Closure, ClosureEnv}, state_term::{StateTerm, StateTermStore}}, State};

pub fn step(
    term: TermPtr,
    mut env: Env,
    mut stack: Stack,
    in_closure: bool,
    mut closure_env: Option<ClosureEnv>
) -> Vec<State>
{
    match term.term() {
        Term::Return(term) => {
            let val = match term.term() {
                Term::Thunk(_) => if in_closure {
                    StateTerm::Closure(Closure {
                        term_ptr: term.clone(), env: closure_env.unwrap()
                    })
                } else {
                    StateTerm::from_term_ptr(term.clone())
                },
                _ => expand_value(term, &env, in_closure, &closure_env)
            };

            match stack.pop() {
                Some(s) => match s {
                    StackTerm::Cont(var, body) => match body {
                        StateTerm::Term(_) => {
                            env.store(var.clone(), val);
                            stack.push(StackTerm::Release(var));

                            vec![State {
                                env,
                                term: body,
                                stack
                            }]
                        },
                        StateTerm::Closure(mut body) => {
                            body.env.store(var, val);

                            vec![State {
                                env,
                                term: StateTerm::Closure(body),
                                stack
                            }]
                        },
                    },
                    StackTerm::Release(var) => {
                        env.release(&var);

                        vec![State {
                            env,
                            term: wrap_return(val),
                            stack
                        }]
                    },
                    StackTerm::Term(_) => unreachable!()
                },
                None => vec![State {
                    env,
                    term: wrap_return(val),
                    stack
                }]
            }
        },
        Term::Bind { var, val, body } => {
            stack.push(StackTerm::Cont(
                var.clone(),
                make_state_term(body.clone(), in_closure, closure_env.clone())
            ));

            vec![State {
                env,
                term: make_state_term(val.clone(), in_closure, closure_env),
                stack
            }]
        },
        Term::Add(lhs, rhs) => {
            let term = TermPtr::from_term(Term::PM(PM::PMNat(PMNat {
                var: lhs.clone(),
                zero: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var(rhs.clone())))),
                succ: PMNatSucc {
                    var: "n".to_string(),
                    body: TermPtr::from_term(Term::Bind {
                        var: "0".to_string(),
                        val: TermPtr::from_term(Term::Add("n".to_string(), rhs.clone())),
                        body: TermPtr::from_term(Term::Return(TermPtr::from_term(
                            Term::Succ(TermPtr::from_term(Term::Var("0".to_string())))
                        )))
                    })
                }
            })));

            vec![State {
                env,
                term: make_state_term(term, in_closure, closure_env),
                stack
            }]
        },
        Term::Eq(lhs, rhs) => {
            let lhs = extract_term(
                lookup(lhs, &env, in_closure, &closure_env)
            );

            let rhs = extract_term(
                lookup(rhs, &env, in_closure, &closure_env)
            );

            vec![State {
                env,
                term: StateTerm::Term(TermPtr::from_term(Term::Return(TermPtr::from_term(
                    Term::Bool(lhs.term() == rhs.term())
                )))),
                stack
            }]
        },
        Term::NEq(lhs, rhs) => {
            let lhs = extract_term(
                lookup(lhs, &env, in_closure, &closure_env)
            );

            let rhs = extract_term(
                lookup(rhs, &env, in_closure, &closure_env)
            );

            vec![State {
                env,
                term: StateTerm::Term(TermPtr::from_term(Term::Return(TermPtr::from_term(
                    Term::Bool(lhs.term() != rhs.term())
                )))),
                stack
            }]
        },
        Term::And(lhs, rhs) => {
            let lhs = match lhs.term() {
                Term::Var(var) => match extract_term(
                    lookup(var, &env, in_closure, &closure_env)
                ).term() {
                    Term::Bool(b) => *b,
                    _ => unreachable!()
                },
                Term::Bool(b) => *b,
                _ => unreachable!()
            };

            let rhs = match rhs.term() {
                Term::Var(var) => match extract_term(
                    lookup(var, &env, in_closure, &closure_env)
                ).term() {
                    Term::Bool(b) => *b,
                    _ => unreachable!()
                },
                Term::Bool(b) => *b,
                _ => unreachable!()
            };

            vec![State {
                env,
                term: StateTerm::Term(TermPtr::from_term(Term::Return(TermPtr::from_term(
                    Term::Bool(lhs && rhs)
                )))),
                stack
            }]
        },
        Term::Or(lhs, rhs) => {
            let lhs = match lhs.term() {
                Term::Var(var) => match extract_term(
                    lookup(var, &env, in_closure, &closure_env)
                ).term() {
                    Term::Bool(b) => *b,
                    _ => unreachable!()
                },
                Term::Bool(b) => *b,
                _ => unreachable!()
            };

            let rhs = match rhs.term() {
                Term::Var(var) => match extract_term(
                    lookup(var, &env, in_closure, &closure_env)
                ).term() {
                    Term::Bool(b) => *b,
                    _ => unreachable!()
                },
                Term::Bool(b) => *b,
                _ => unreachable!()
            };

            vec![State {
                env,
                term: StateTerm::Term(TermPtr::from_term(Term::Return(TermPtr::from_term(
                    Term::Bool(lhs || rhs)
                )))),
                stack
            }]
        },
        Term::Not(var) => match extract_term(lookup(var, &env, in_closure, &closure_env)).term() {
            Term::Bool(b) => vec![State {
                env,
                term: StateTerm::Term(TermPtr::from_term(Term::Return(TermPtr::from_term(
                    Term::Bool(!b)
                )))),
                stack
            }],
            _ => unreachable!()
        },
        Term::If { cond, then, r#else } => {
            let cond = match extract_term(lookup(cond, &env, in_closure, &closure_env)).term() {
                Term::Bool(b) => *b,
                _ => unreachable!()
            };

            vec![State {
                env,
                term: StateTerm::Term(if cond {
                    then.clone()
                } else {
                    r#else.clone()
                }),
                stack
            }]
        },
        Term::App(lhs, rhs) => {
            stack.push(StackTerm::Term(
                lookup(&rhs, &env, in_closure, &closure_env)
            ));

            vec![State {
                env,
                term: make_state_term(lhs.clone(), in_closure, closure_env),
                stack
            }]
        },
        Term::Force(var) => {
            let term = match lookup(var, &env, in_closure, &closure_env) {
                StateTerm::Term(term) => match term.term() {
                    Term::Thunk(term) => StateTerm::from_term_ptr(term.clone()),
                    _ => unreachable!()
                },
                StateTerm::Closure(closure) => match closure.term_ptr.term() {
                    Term::Thunk(term) => StateTerm::Closure(Closure {
                        term_ptr: term.clone(), env: closure.env
                    }),
                    _ => unreachable!()
                }
            };

            vec![State {
                env, term, stack
            }]
        },
        Term::Lambda { arg, body } => match stack.pop().unwrap() {
            StackTerm::Term(term) => {
                let mut closure = if in_closure {
                    Closure {
                        term_ptr: body.clone(),
                        env: closure_env.unwrap()
                    }
                } else {
                    Closure::from_term_ptr(body.clone())
                };

                closure.store_arg(arg.clone(), term);

                vec![State {
                    env,
                    term: StateTerm::Closure(closure),
                    stack
                }]
            },
            _ => unreachable!()
        },
        Term::PM(pm) => match pm {
            PM::PMNat(pm_nat) => match extract_term(lookup(&pm_nat.var, &env, in_closure, &closure_env)).term() {
                Term::Zero => vec![State {
                    env,
                    term: make_state_term(pm_nat.zero.clone(), in_closure, closure_env),
                    stack
                }],
                Term::Succ(s) => {
                    if !in_closure {
                        stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                    }
                    store(pm_nat.succ.var.clone(), StateTerm::from_term_ptr(s.clone()), &mut env, in_closure, &mut closure_env);

                    vec![State {
                        env,
                        term: make_state_term(pm_nat.succ.body.clone(), in_closure, closure_env),
                        stack
                    }]
                },
                Term::TypedVar(shape) => if shape.borrow().is_some() {
                    match shape.borrow().as_ref().unwrap().term() {
                        Term::Zero => vec![State {
                            env,
                            term: make_state_term(pm_nat.zero.clone(), in_closure, closure_env),
                            stack
                        }],
                        Term::Succ(s) => {
                            if !in_closure {
                                stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                            }
                            store(pm_nat.succ.var.clone(), StateTerm::from_term_ptr(s.clone()), &mut env, in_closure, &mut closure_env);

                            vec![State {
                                env,
                                term: make_state_term(pm_nat.succ.body.clone(), in_closure, closure_env),
                                stack
                            }]
                        },
                        _ => unreachable!()
                    }
                } else {
                    vec![
                        {    
                            let mut new_val_locs = HashMap::new();
                            let mut new_env_locs = HashMap::new();

                            let env = env.clone_with_locations(&mut new_val_locs, &mut new_env_locs);
                            let closure_env = match &closure_env {
                                Some(closure_env) => Some(closure_env.clone_with_locations(&mut new_val_locs, &mut new_env_locs)),
                                None => None
                            };
                            let stack = stack.clone_with_locations(&mut new_val_locs, &mut new_env_locs);

                            let shape = match lookup(&pm_nat.var, &env, in_closure, &closure_env) {
                                StateTerm::Term(term_ptr) => match term_ptr.term() {
                                    Term::TypedVar(shape) => Rc::clone(shape),
                                    _ => unreachable!()
                                },
                                StateTerm::Closure(_) => unreachable!()
                            };

                            shape.replace(Some(TermPtr::from_term(Term::Zero)));

                            State {
                                env,
                                term: make_state_term(pm_nat.zero.clone(), in_closure, closure_env),
                                stack
                            }
                        },
                        {
                            let s = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                            shape.replace(Some(TermPtr::from_term(Term::Succ(s.clone()))));

                            if !in_closure {
                                stack.push(StackTerm::Release(pm_nat.succ.var.clone()));
                            }
                            store(pm_nat.succ.var.clone(), StateTerm::from_term_ptr(s), &mut env, in_closure, &mut closure_env);

                            State {
                                env,
                                term: make_state_term(pm_nat.succ.body.clone(), in_closure, closure_env),
                                stack
                            }
                        }
                    ]
                },
                _ => unreachable!()
            },
            PM::PMList(pm_list) => match extract_term(lookup(&pm_list.var, &env, in_closure, &closure_env)).term() {
                Term::Nil => vec![State {
                    env,
                    term: make_state_term(pm_list.nil.clone(), in_closure, closure_env),
                    stack
                }],
                Term::Cons(x, xs) => {
                    if !in_closure {
                        stack.push(StackTerm::Release(pm_list.cons.x.clone()));
                    }
                    store(pm_list.cons.x.clone(), StateTerm::from_term_ptr(x.clone()), &mut env, in_closure, &mut closure_env);

                    if !in_closure {
                        stack.push(StackTerm::Release(pm_list.cons.xs.clone()));
                    }
                    store(pm_list.cons.xs.clone(), StateTerm::from_term_ptr(xs.clone()), &mut env, in_closure, &mut closure_env);

                    vec![State {
                        env,
                        term: make_state_term(pm_list.cons.body.clone(), in_closure, closure_env),
                        stack
                    }]
                },
                Term::TypedVar(shape) => if shape.borrow().is_some() {
                    match shape.borrow().as_ref().unwrap().term() {
                        Term::Nil => vec![State {
                            env,
                            term: make_state_term(pm_list.nil.clone(), in_closure, closure_env),
                            stack
                        }],
                        Term::Cons(x, xs) => {
                            if !in_closure {
                                stack.push(StackTerm::Release(pm_list.cons.x.clone()));
                            }
                            store(pm_list.cons.x.clone(), StateTerm::from_term_ptr(x.clone()), &mut env, in_closure, &mut closure_env);

                            if !in_closure {
                                stack.push(StackTerm::Release(pm_list.cons.xs.clone()));
                            }
                            store(pm_list.cons.xs.clone(), StateTerm::from_term_ptr(xs.clone()), &mut env, in_closure, &mut closure_env);

                            vec![State {
                                env,
                                term: make_state_term(pm_list.cons.body.clone(), in_closure, closure_env),
                                stack
                            }]
                        },
                        _ => unreachable!()
                    }
                } else {
                    vec![
                        {
                            let mut new_val_locs = HashMap::new();
                            let mut new_env_locs = HashMap::new();

                            let env = env.clone_with_locations(&mut new_val_locs, &mut new_env_locs);
                            let closure_env = match &closure_env {
                                Some(closure_env) => Some(closure_env.clone_with_locations(&mut new_val_locs, &mut new_env_locs)),
                                None => None
                            };
                            let stack = stack.clone_with_locations(&mut new_val_locs, &mut new_env_locs);

                            let shape = match lookup(&pm_list.var, &env, in_closure, &closure_env) {
                                StateTerm::Term(term_ptr) => match term_ptr.term() {
                                    Term::TypedVar(shape) => Rc::clone(shape),
                                    _ => unreachable!()
                                },
                                StateTerm::Closure(_) => unreachable!()
                            };

                            shape.replace(Some(TermPtr::from_term(Term::Nil)));

                            State {
                                env,
                                term: make_state_term(pm_list.nil.clone(), in_closure, closure_env.clone()),
                                stack
                            }
                        },
                        {
                            let x = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                            let xs = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                            shape.replace(Some(TermPtr::from_term(Term::Cons(x.clone(), xs.clone()))));

                            if !in_closure {
                                stack.push(StackTerm::Release(pm_list.cons.x.clone()));
                            }
                            store(pm_list.cons.x.clone(), StateTerm::from_term_ptr(x), &mut env, in_closure, &mut closure_env);

                            if !in_closure {
                                stack.push(StackTerm::Release(pm_list.cons.xs.clone()));
                            }
                            store(pm_list.cons.xs.clone(), StateTerm::from_term_ptr(xs), &mut env, in_closure, &mut closure_env);

                            State {
                                env,
                                term: make_state_term(pm_list.cons.body.clone(), in_closure, closure_env),
                                stack
                            }
                        }
                    ]
                },
                _ => unreachable!()
            }
        },
        Term::Choice(choices) => choices.into_iter()
            .map(|choice| {
                let mut new_val_locs = HashMap::new();
                let mut new_env_locs = HashMap::new();

                State {
                    env: env.clone_with_locations(&mut new_val_locs, &mut new_env_locs),
                    term: make_state_term(choice.clone(), in_closure, closure_env.clone()),
                    stack: stack.clone_with_locations(&mut new_val_locs, &mut new_env_locs)
                }
            }).collect(),
        Term::Exists { var, body } => {
            store(
                var.clone(),
                StateTerm::from_term(Term::TypedVar(Rc::new(RefCell::new(None)))),
                &mut env, in_closure, &mut closure_env
            );

            vec![State {
                env,
                term: make_state_term(body.clone(), in_closure, closure_env),
                stack
            }]
        },
        Term::Equate { lhs, rhs, body } => {
            let lhs = extract_term(
                lookup(lhs, &env, in_closure, &closure_env)
            );

            let rhs = extract_term(
                lookup(rhs, &env, in_closure, &closure_env)
            );

            let flag = equate(lhs, rhs);

            vec![State {
                env,
                term: if flag {
                    make_state_term(body.clone(), in_closure, closure_env)
                } else {
                    StateTerm::from_term(Term::Fail)
                },
                stack
            }]
        },
        Term::Fail => vec![State {
            env,
            term: StateTerm::from_term(Term::Fail),
            stack
        }],
        _ => unreachable!()
    }
}

fn make_state_term(term: TermPtr, in_closure: bool, closure_env: Option<ClosureEnv>) -> StateTerm {
    if in_closure {
        StateTerm::Closure(Closure {
            term_ptr: term, env: closure_env.unwrap()
        })
    } else {
        StateTerm::Term(term)
    }
}

fn expand_value(term: &TermPtr, env: &Env, in_closure: bool, closure_env: &Option<ClosureEnv>) -> StateTerm {
    match term.term() {
        Term::Var(var) => lookup(var, env, in_closure, closure_env),
        Term::Pair(lhs, rhs) => match expand_value(lhs, env, in_closure, closure_env) {
            StateTerm::Term(lhs) => match expand_value(rhs, env, in_closure, closure_env) {
                StateTerm::Term(rhs) => StateTerm::from_term(Term::Pair(lhs, rhs)),
                StateTerm::Closure(_) => unreachable!()
            },
            StateTerm::Closure(_) => unreachable!()
        },
        Term::Succ(term_ptr) => match expand_value(term_ptr, env, in_closure, closure_env) {
            StateTerm::Term(term_ptr) => StateTerm::from_term(Term::Succ(term_ptr)),
            StateTerm::Closure(_) => unreachable!()
        },
        Term::Cons(x, xs) => match expand_value(x, env, in_closure, closure_env) {
            StateTerm::Term(x) => match expand_value(xs, env, in_closure, closure_env) {
                StateTerm::Term(xs) => StateTerm::from_term(Term::Cons(x, xs)),
                StateTerm::Closure(_) => unreachable!()
            },
            StateTerm::Closure(_) => unreachable!()
        }
        Term::TypedVar(shape) => match shape.borrow().as_ref() {
            Some(term_ptr) => match term_ptr.term() {
                Term::Zero => StateTerm::from_term(Term::Zero),
                Term::Succ(term_ptr) => match expand_value(term_ptr, env, in_closure, closure_env) {
                    StateTerm::Term(term_ptr) => StateTerm::from_term(Term::Succ(term_ptr)),
                    StateTerm::Closure(_) => unreachable!()
                },
                Term::Nil => StateTerm::from_term(Term::Nil),
                Term::Cons(x, xs) => match expand_value(x, env, in_closure, closure_env) {
                    StateTerm::Term(x) => match expand_value(xs, env, in_closure, closure_env) {
                        StateTerm::Term(xs) => StateTerm::from_term(Term::Cons(x, xs)),
                        StateTerm::Closure(_) => unreachable!()
                    },
                    StateTerm::Closure(_) => unreachable!()
                }
                _ => unreachable!()
            },
            None => StateTerm::from_term(Term::TypedVar(Rc::clone(shape)))
        },
        _ => StateTerm::from_term_ptr(term.clone())
    }
}

fn wrap_return(val: StateTerm) -> StateTerm {
    match val {
        StateTerm::Term(term) => StateTerm::from_term(Term::Return(term)),
        StateTerm::Closure(closure) => StateTerm::Closure(Closure {
            term_ptr: TermPtr::from_term(Term::Return(closure.term_ptr)), env: closure.env
        })
    }
}

fn lookup(var: &String, env: &Env, in_closure: bool, closure_env: &Option<ClosureEnv>) -> StateTerm {
    if in_closure {
        match closure_env.as_ref().unwrap().lookup(var) {
            Some(val) => val,
            None => env.lookup(var).unwrap()
        }
    } else {
        env.lookup(var).unwrap()
    }
}

fn store(var: String, val: StateTerm, env: &mut Env, in_closure: bool, closure_env: &mut Option<ClosureEnv>) {
    if in_closure {
        closure_env.as_mut().unwrap().store(var, val);
    } else {
        env.store(var, val);
    }
}

fn extract_term(val: StateTerm) -> TermPtr {
    match val {
        StateTerm::Term(term) => term,
        StateTerm::Closure(_) => unreachable!()
    }
}
