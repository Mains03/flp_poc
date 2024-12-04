use std::{borrow::Borrow, cell::RefCell, collections::{HashMap, VecDeque}, ptr, rc::Rc};
use crate::{cbpv::terms::{Value, ValueType}, machine::{lvar, senv}};
use super::{lvar::LogicEnv, mterms::{MComputation, MValue}, senv::SuspEnv, unify::UnifyError, Env, Ident, VClosure};
use crate::machine::unify::unify;
    
#[derive(Debug)]
enum Frame {
    Value(Rc<MValue>),
    To(Rc<MComputation>),
    Set(Ident, Rc<MComputation>)
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

fn push_susp(stack : &Stack, ident : Ident, c : Rc<MComputation>, env:  Rc<Env>) -> Rc<Stack> {
    push_closure(stack, Frame::Set(ident, c), env)
}

#[derive(Clone)]
pub struct Machine {
    pub comp : Rc<MComputation>,
    pub env  : Rc<Env>,
    pub lenv : LogicEnv,
    pub senv : SuspEnv,
    pub stack : Rc<Stack>,
    pub done : bool
}

impl Machine {
    pub fn step(self) -> Vec<Machine> {
        let m = self;
        
        match &*(m.comp) {
            MComputation::Return(val) => {
                match &*(m.stack).as_slice() {
                    [] => {
                        match (VClosure::Clos { val : val.clone(), env: m.env.clone() }).find_susp(&m.lenv, &m.senv) {
                            Some(a) => vec![Machine { comp : a.comp, env : a.env, stack : push_susp(&m.stack, a.ident, m.comp, m.env), ..m  }],
                            None => vec![Machine { done: true, ..m }]
                        }
                    },
                    [tail @ .., clos] => {
                        let Closure { frame , env } = &*clos;
                        match &**frame {
                            Frame::Value(_) => unreachable!("return throws value to a value"),
                            Frame::To(cont) => {
                                let new_env = env.extend_clos(val.clone(), m.env.clone());
                                vec![Machine { comp: cont.clone(), stack : Rc::new(tail.to_vec()), env : new_env, ..m }]
                            },
                            Frame::Set(i, cont) => {
                                let mut senv = m.senv;
                                senv.set(i, val.clone(), m.env);
                                vec![Machine { comp: cont.clone(), stack : Rc::new(tail.to_vec()), env : env.clone(), senv : senv, ..m }]
                            },
                        }
                    },
                      _ => unreachable!()
                  }
            },
            MComputation::Bind { comp, cont } => {
                match &**comp {
                    MComputation::Return(v) => {
                        let new_env = m.env.extend_clos(v.clone(), m.env.clone());
                        vec![Machine { comp : cont.clone(), env : new_env, ..m }]
                    },
                    _ => {
                        let mut senv = m.senv;
                        let env = &m.env;
                        let ident = senv.fresh(comp, &m.env);
                        let new_env = env.extend_susp(ident);
                        vec![Machine { comp : cont.clone(), env : new_env, senv : senv, ..m}]
                    }
                }
            },
            MComputation::Force(v) => {
                let vclos = VClosure::Clos { val: v.clone(), env: m.env.clone() };
                match vclos.close_head(&m.lenv, &m.senv){
                    Ok(vclos) => 
                        match &*vclos {
                            VClosure::Clos { val, env } => {
                                match &**val {
                                    MValue::Thunk(t) => vec![Machine { comp : t.clone(), env : env.clone(), ..m}],
                                _ => panic!("shouldn't be forcing a non-thunk value")
                                } 
                            },
                            VClosure::LogicVar { ident } => panic!("shouldn't be forcing a logic variable"),
                            VClosure::Susp { ident } => unreachable!("shouldn't be forcing a suspension"),
                        }
                    Err(a) => {
                        vec![Machine { comp : a.comp, env : a.env, stack : push_susp(&m.stack, a.ident, m.comp, m.env), ..m  }]
                    },
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
                let mut lenv = m.lenv;
                let ident = lenv.fresh(ptype.clone());
                vec![Machine { comp : body.clone(), env : m.env.extend_lvar(ident), lenv : lenv, ..m}]
            }
            MComputation::Equate { lhs, rhs, body } => {
                let mut lenv = m.lenv;
                let mut senv = m.senv;
                match unify(lhs, rhs, &m.env, &mut lenv, &senv) {
                    Ok(()) => vec![Machine {comp : body.clone(), lenv : lenv, senv : senv, ..m }],
                    Err(UnifyError::Susp(a)) => {
                        vec![Machine { comp : a.comp, env : a.env, stack : push_susp(&m.stack, a.ident, m.comp, m.env), lenv : lenv, senv : senv, ..m  }]
                    }
                    Err(_) => vec![]
                }
            },
            MComputation::Ifz { num, zk, sk } => {
                let vclos = VClosure::Clos { val : num.clone(), env: m.env.clone() };
                let closed_num = vclos.close_head(&m.lenv, &m.senv);
                match closed_num {
                    Err(a) => vec![Machine { comp : a.comp, env : a.env, stack : push_susp(&m.stack, a.ident, m.comp, m.env), ..m  }],
                    Ok(vclos) => {
                        match &*vclos {
                            VClosure::Clos { val, env } => {
                                match &**val {
                                    MValue::Zero => vec![Machine { comp: zk.clone(), ..m}],
                                    MValue::Succ(v) => {
                                        let new_menv = m.env.extend_clos(v.clone(), env.clone());
                                        vec![Machine { comp: sk.clone(), env : new_menv, ..m}]
                                    }
                                    _ => panic!("Ifz on something non-numerical")
                                }
                            },
                            VClosure::LogicVar { ident } => {  // must be unresolved, by structure of close_head
                                let m_zero  = {
                                    let mut lenv = m.lenv.clone(); // make a new logic env
                                    lenv.set_vclos(&ident, &VClosure::Clos {
                                       val: MValue::Zero.into(), 
                                       env: Env::empty() 
                                    }.into());

                                    Machine { comp: zk.clone(), lenv : lenv, ..m.clone()}
                                };
                                
                                let m_succ = {
                                    let mut lenv = m.lenv.clone();
                                    let ident_lvar_succ = lenv.fresh(ValueType::Nat);
                                    
                                    lenv.set_vclos(&ident, &VClosure::Clos { 
                                        val : MValue::Succ(Rc::new(MValue::Var(0))).into(), 
                                        env : Env::empty().extend_lvar(ident_lvar_succ)
                                    }.into());
                                    
                                    let new_env = m.env.extend_lvar(ident_lvar_succ);

                                    Machine { comp: sk.clone(), lenv : lenv, env : new_env, ..m.clone()}
                                };

                                vec![m_zero, m_succ]
                            }
                            VClosure::Susp { ident } => unreachable!("shouldn't")
                        }
                    }
                }
            },
            MComputation::Match { list, nilk, consk } => {
                let vclos = VClosure::Clos { val : list.clone(), env: m.env.clone() };
                let closed_list = vclos.close_head(&m.lenv, &m.senv);
                match closed_list {
                    Err(a) => vec![Machine { comp : a.comp, env : a.env, stack : push_susp(&m.stack, a.ident, m.comp, m.env), ..m  }],
                    Ok(vclos) => 
                        match &*vclos {
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
                            VClosure::LogicVar { ident } => {  // must be unresolved, by structure of close_head
                                                              
                                let ptype = match m.lenv.get_type(&ident) {
                                    ValueType::List(t) => t,
                                    _ => panic!("matching on a non-list logical variable")
                                };

                                let m_nil  = {
                                    
                                    let mut lenv = m.lenv.clone();
                                    lenv.set_vclos(&ident, &VClosure::Clos { val: MValue::Nil.into(), env: Env::empty() }.into());

                                    Machine { comp: nilk.clone(), lenv : lenv, ..m.clone()}
                                };
                                
                                let m_cons = {
                                    
                                    let mut lenv = m.lenv.clone();
                                    let head_ident = lenv.fresh(*ptype.clone());
                                    let tail_ident = lenv.fresh(ValueType::List(ptype));
                                    
                                    lenv.set_vclos(&ident, &VClosure::Clos {
                                        val: MValue::Cons(Rc::new(MValue::Var(1)), Rc::new(MValue::Var(0))).into(),
                                        env: Env::empty().extend_lvar(head_ident).extend_lvar(tail_ident)
                                    }.into());
                                    
                                    let env = m.env.extend_lvar(head_ident).extend_lvar(tail_ident);

                                    Machine { comp: consk.clone(), lenv : lenv, env : env, ..m.clone()}
                                };
                                vec![m_nil, m_cons]
                            }
                            VClosure::Susp { ident } => unreachable!("nonono"),
                        }
                }
            },
            MComputation::Case { sum, inlk, inrk } => {
                let vclos = VClosure::Clos { val : sum.clone(), env: m.env.clone() };
                let closed_sum = vclos.close_head(&m.lenv, &m.senv);
                match closed_sum {
                    Err(a) => vec![Machine { comp : a.comp, env : a.env, stack : push_susp(&m.stack, a.ident, m.comp, m.env), ..m  }],
                    Ok(vclos) => 
                        match &*vclos {
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
                            VClosure::LogicVar { ident } => {  // must be unresolved, by structure of close_head
                                                              
                                let (ptype1, ptype2) = match m.lenv.get_type(&ident) {
                                    ValueType::Sum(t1, t2) => (t1, t2),
                                    _ => panic!("case-ing on a non-sum logical variable")
                                };

                                let m_inl = {
                                    // let env = m.env.deep_clone();
                                    // let vclos = VClosure::Clos { val : sum.clone(), env: env.clone() };
                                    // let closed = vclos.close_head(); // re-find lvar in deep clone
                                    // if let VClosure::LogicVar { lvar } = &*closed {
                                    //     // make a new lvar of inl type, and stick it into the new machine
                                    //     let lvar_inl = LogicVar::new(*(ptype1.clone()));
                                    //     let mut new_env = Env::empty().extend_lvar(lvar_inl);
                                    //     lvar.set_val(MValue::Inl(Rc::new(MValue::Var(0))).into(), new_env)
                                    // }
                                    // Machine { comp: inlk.clone(), env, ..m.clone()}
                                    todo!()
                                };

                                let m_inr = {
                                    // let env = m.env.deep_clone();
                                    // let vclos = VClosure::Clos { val : sum.clone(), env: env.clone() };
                                    // let closed = vclos.close_head(); // re-find lvar in deep clone
                                    // if let VClosure::LogicVar { lvar } = &*closed {
                                    //     // make a new lvar of inl type, and stick it into the new machine
                                    //     let lvar_inr = LogicVar::new(*(ptype2.clone()));
                                    //     let mut new_env = Env::empty().extend_lvar(lvar_inr);
                                    //     lvar.set_val(MValue::Inl(Rc::new(MValue::Var(0))).into(), new_env)
                                    // }
                                    // Machine { comp: inlk.clone(), env, ..m.clone()}
                                    todo!()
                                };
                                
                                vec![m_inl, m_inr]
                            }
                            VClosure::Susp { ident } => unreachable!("oops")
                        }
                }
            },
            MComputation::Rec { body } => {
                let new_env = m.env.extend_clos(MValue::Thunk(m.comp.clone()).into(), m.env.clone());
                vec![Machine { comp : body.clone(), env : new_env, ..m }] 
            },
        }
    }
    
}