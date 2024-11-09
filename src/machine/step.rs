use std::{borrow::Borrow, cell::RefCell, collections::{HashMap, VecDeque}, ptr, rc::Rc};
use crate::cbpv::terms::ValueType;
use super::{lvar::LogicVar, mterms::{MComputation, MValue}, Env, VClosure};
use crate::machine::unify::unify;
    

#[derive(Debug)]
enum Frame {
    Value(Rc<MValue>),
    To(Rc<MComputation>)
}

#[derive(Clone, Debug)]
pub struct Closure {
    frame : Rc<Frame>,
    env : Rc<Env>
}

type Stack = Vec<Closure>;

pub fn empty_stack() -> Stack { vec![] }

fn push_closure(stack : &Stack, frame : Frame, env : Rc<Env>) -> Rc<Stack> {
    let mut stk = stack.clone();
    stk.push(Closure { frame: frame.into(), env });
    Rc::new(stk)
}

#[derive(Clone)]
pub struct Machine {
    pub comp : Rc<MComputation>,
    pub env  : Rc<Env>,
    pub stack : Rc<Stack>,
    pub done : bool
}

pub fn step(m : Machine) -> Vec<Machine> {
    match &*(m.comp) {
        MComputation::Return(val) => {
            match &*(m.stack).as_slice() {
                [] => vec![Machine { done: true, ..m }],
                [tail @ .., clos] => {
                    let Closure { frame , env } = &*clos;
                    if let Frame::To(cont) = &**frame {
                        let new_env = env.extend_clos(val.clone(), m.env.clone());
                        vec![Machine { comp: cont.clone(), stack : Rc::new(tail.to_vec()), env : new_env, ..m }]
                    } else { panic!("return but no to frame in the stack") }
                },
                  _ => unreachable!()
              }
        },
        MComputation::Bind { comp, cont } => 
            vec![Machine { comp: comp.clone(), stack: push_closure(&m.stack, Frame::To(cont.clone()), m.env.clone()), ..m}],
        MComputation::Force(v) => {
            let w = VClosure::Clos { val: v.clone(), env: m.env.clone() }.close_head();
            match &*w {
                VClosure::Clos { val, env } => {
                    match &**val {
                        MValue::Thunk(t) => vec![Machine { comp : t.clone(), env : env.clone(), ..m}],
                    _ => panic!("shouldn't be forcing a non-thunk value")
                    } 
                },
                VClosure::LogicVar { lvar } => panic!("shouldn't be forcing a logic variable"),
            }
        },
        MComputation::Lambda { body } => {
            match &*(m.stack).as_slice() {
                [] => panic!("lambda met with empty stack"),
                [tail @ .., clos] => {
                    let Closure { frame , env} = &*clos;
                    if let Frame::Value(val) = &**frame {
                        let new_env = m.env.extend_clos(val.clone(), env.clone());
                        vec![Machine { comp: body.clone(), stack: Rc::new(tail.to_vec()), env : new_env, ..m}]
                    } else { panic!("lambda but no value frame in the stack") }
                },
                _ => unreachable!()
              }
        },
        MComputation::App { op, arg } => 
            vec![Machine { comp: op.clone(), stack: push_closure(&m.stack, Frame::Value(arg.clone()), m.env.clone()), ..m}],
        MComputation::Choice(choices) => 
          choices.iter().map(|c| Machine { comp: c.clone(), ..m.clone()}).collect(),
        MComputation::Exists { ptype, body } => {
            vec![Machine { comp : body.clone(), env : m.env.extend_lvar(LogicVar::new(ptype.clone())), ..m}]
        }
        MComputation::Equate { lhs, rhs, body } => {
          if unify(lhs, rhs, &m.env) {
            vec![Machine {comp : body.clone(), ..m }]
          } else {
            vec![]
          }
        },
        MComputation::Ifz { num, zk, sk } => {
            let vclos = VClosure::Clos { val : num.clone(), env: m.env.clone() };
            let closed_num = vclos.close_head();
            match &*closed_num {
                VClosure::Clos { val, env } => {
                    match &**val {
                        MValue::Zero => vec![Machine { comp: zk.clone(), ..m}],
                        MValue::Succ(_) => vec![Machine { comp: sk.clone(), ..m}],
                        _ => panic!("Ifz on something non-numerical")
                    }
                },
                VClosure::LogicVar { .. } => {  // must be unresolved, by structure of close_head
                    let m_zero  = {
                        let env_zero = m.env.deep_clone();
                        let vclos_zero = VClosure::Clos { val : num.clone(), env: env_zero.clone() };
                        let closed_num_zero = vclos_zero.close_head();
                        if let VClosure::LogicVar { lvar } = &*closed_num_zero {
                            lvar.set_val(MValue::Zero.into(), Env::empty())
                        }
                        else { unreachable!() } 

                        Machine { comp: zk.clone(), env : env_zero, ..m.clone()}
                    };
                    
                    let m_succ = {
                        let lvar_succ = LogicVar::new(ValueType::Nat);
                        let mut lvar_env = Env::empty();
                        lvar_env.extend_lvar(lvar_succ);

                        let env_succ = m.env.deep_clone();
                        let vclos_succ = VClosure::Clos { val : num.clone(), env: env_succ.clone() };
                        let closed_num_succ = vclos_succ.close_head();
                        if let VClosure::LogicVar { lvar } = &*closed_num_succ {
                            lvar.set_val(MValue::Succ(Rc::new(MValue::Var(0))).into(), lvar_env.into())
                        }
                        else { unreachable!() } 

                        Machine { comp: sk.clone(), env : env_succ, ..m.clone()}
                    };

                    vec![m_zero, m_succ]
                }
            }
        },
        MComputation::Match { list, nilk, consk } => {
            let vclos = VClosure::Clos { val : list.clone(), env: m.env.clone() };
            let closed_list = vclos.close_head();
            match &*closed_list {
                VClosure::Clos { val, env } => {
                    match &**val {
                        MValue::Nil => vec![Machine { comp: nilk.clone(), ..m}],
                        MValue::Cons(v, w) => {
                            let new_menv = 
                                m.env.extend_clos(v.clone(), env.clone()).extend_clos(w.clone(), env.clone());
                            vec![Machine { comp: consk.clone(), env : new_menv, ..m}]
                        },
                        _ => panic!("Match on non-list")
                    }
                },
                VClosure::LogicVar { lvar } => {  // must be unresolved, by structure of close_head
                                                  
                    let ptype = match lvar.get_type() {
                        ValueType::List(t) => t,
                        _ => panic!("matching on a non-list logical variable")
                    };

                    let m_nil  = {

                        println!("[DEBUG] about to deep clone");
                        let env_nil = m.env.deep_clone();
                        println!("[DEBUG] just deep cloned");
                        let vclos_nil = VClosure::Clos { val : list.clone(), env: env_nil.clone() };
                        let closed_list_nil = vclos_nil.close_head();
                        if let VClosure::LogicVar { lvar } = &*closed_list_nil {
                            lvar.set_val(MValue::Nil.into(), Env::empty())
                        }
                        else { unreachable!("closure was returned when closure shouldn't be returned") } 

                        // println!("env_nil: {:?}", env_nil.iter().map(|vclos| vclos.val()).collect::<Vec<String>>());
                        Machine { comp: nilk.clone(), env : env_nil, ..m.clone()}
                    };
                    
                    let m_cons = {
                        let lvar_head = LogicVar::new(*(ptype.clone()));
                        let lvar_tail = LogicVar::new(ValueType::List(ptype.clone()));
                        let lvar_env = Env::empty().extend_lvar(lvar_head.clone()).extend_lvar(lvar_tail.clone());

                        let env_cons = m.env.deep_clone();
                        let vclos_cons = VClosure::Clos { val : list.clone(), env: env_cons.clone() };
                        let closed_num_cons = vclos_cons.close_head();
                        if let VClosure::LogicVar { lvar } = &*closed_num_cons {
                            lvar.set_val(MValue::Cons(Rc::new(MValue::Var(1)), Rc::new(MValue::Var(0))).into(), lvar_env)
                        }
                        else { unreachable!("closure was returned when closure shouldn't be returned: {:?}", vclos_cons.close_head().val() ) } 
                        let final_env = env_cons.extend_lvar(lvar_head).extend_lvar(lvar_tail);
                        
                        // println!("env_cons: {:?}", env_cons.iter().map(|vclos| vclos.val()).collect::<Vec<String>>());
                        Machine { comp: consk.clone(), env : final_env, ..m.clone()}
                    };
                    vec![m_nil, m_cons]
                }
            }
        },
        MComputation::Case { sum, inlk, inrk } => {
            let vclos = VClosure::Clos { val : sum.clone(), env: m.env.clone() };
            let closed_sum = vclos.close_head();
            match &*closed_sum {
                VClosure::Clos { val, env } => {
                    match &**val {
                        MValue::Inl(v) => {
                            let old_env = env.clone();
                            let new_env = m.env.extend_clos(v.clone(), old_env.clone());
                            vec![Machine { comp: inlk.clone(), env : new_env, ..m}]
                        },
                        MValue::Inr(v) => {
                            let old_env = env.clone();
                            let new_env = m.env.extend_clos(v.clone(), old_env.clone());
                            vec![Machine { comp: inrk.clone(), env : new_env, ..m}]
                        },
                        _ => panic!("Match on non-list")
                    }
                },
                VClosure::LogicVar { lvar } => {  // must be unresolved, by structure of close_head
                                                  
                    let (ptype1, ptype2) = match lvar.get_type() {
                        ValueType::Sum(t1, t2) => (t1, t2),
                        _ => panic!("case-ing on a non-sum logical variable")
                    };

                    let m_inl = {
                        let env = m.env.deep_clone();
                        let vclos = VClosure::Clos { val : sum.clone(), env: env.clone() };
                        let closed = vclos.close_head(); // re-find lvar in deep clone
                        if let VClosure::LogicVar { lvar } = &*closed {
                            // make a new lvar of inl type, and stick it into the new machine
                            let lvar_inl = LogicVar::new(*(ptype1.clone()));
                            let mut new_env = Env::empty().extend_lvar(lvar_inl);
                            lvar.set_val(MValue::Inl(Rc::new(MValue::Var(0))).into(), new_env)
                        }
                        Machine { comp: inlk.clone(), env, ..m.clone()}
                    };

                    let m_inr = {
                        let env = m.env.deep_clone();
                        let vclos = VClosure::Clos { val : sum.clone(), env: env.clone() };
                        let closed = vclos.close_head(); // re-find lvar in deep clone
                        if let VClosure::LogicVar { lvar } = &*closed {
                            // make a new lvar of inl type, and stick it into the new machine
                            let lvar_inr = LogicVar::new(*(ptype2.clone()));
                            let mut new_env = Env::empty().extend_lvar(lvar_inr);
                            lvar.set_val(MValue::Inl(Rc::new(MValue::Var(0))).into(), new_env)
                        }
                        Machine { comp: inlk.clone(), env, ..m.clone()}
                    };
                    
                    vec![m_inl, m_inr]
                }
            }
        },
        MComputation::Rec { body } => {
            let new_env = m.env.extend_clos(MValue::Thunk(m.comp.clone()).into(), m.env.clone());
            vec![Machine { comp : body.clone(), env : new_env, ..m }] 
        },
    }
}