use std::{borrow::Borrow, cell::RefCell, collections::{HashMap, VecDeque}, ptr, rc::Rc};
use crate::{cbpv::terms::{Value, ValueType}, machine::lvar};
use super::{lvar::LogicEnv, mterms::{MComputation, MValue}, unify::UnifyError, Env, VClosure};
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
    pub lenv : LogicEnv,
    pub stack : Rc<Stack>,
    pub done : bool
}

impl Machine {
    pub fn step(self) -> Vec<Machine> {
        let m = self;
        
        if m.will_necessarily_fail() { return vec![] };

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
                let vclos = VClosure::Clos { val: v.clone(), env: m.env.clone() };
                let w = vclos.close_head(&m.lenv);
                match &*w {
                    VClosure::Clos { val, env } => {
                        match &**val {
                            MValue::Thunk(t) => vec![Machine { comp : t.clone(), env : env.clone(), ..m}],
                        _ => panic!("shouldn't be forcing a non-thunk value")
                        } 
                    },
                    VClosure::LogicVar { ident } => panic!("shouldn't be forcing a logic variable"),
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
                match unify(lhs, rhs, &m.env, &mut lenv) {
                    Ok(()) => vec![Machine {comp : body.clone(), lenv : lenv, ..m }],
                    Err(_) => vec![]
                }
            },
            MComputation::Ifz { num, zk, sk } => {
                let vclos = VClosure::Clos { val : num.clone(), env: m.env.clone() };
                let closed_num = vclos.close_head(&m.lenv);
                match &*closed_num {
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
                }
            },
            MComputation::Match { list, nilk, consk } => {
                let vclos = VClosure::Clos { val : list.clone(), env: m.env.clone() };
                let closed_list = vclos.close_head(&m.lenv);
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
                }
            },
            MComputation::Case { sum, inlk, inrk } => {
                let vclos = VClosure::Clos { val : sum.clone(), env: m.env.clone() };
                let closed_sum = vclos.close_head(&m.lenv);
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
                }
            },
            MComputation::Rec { body } => {
                let new_env = m.env.extend_clos(MValue::Thunk(m.comp.clone()).into(), m.env.clone());
                vec![Machine { comp : body.clone(), env : new_env, ..m }] 
            },
        }
    }
    
    fn will_necessarily_fail(&self) -> bool {
        let mut c = &self.comp;
        let mut lenv = self.lenv.clone();
        let mut env = self.env.clone();
        loop {
            c = match &**c {
                MComputation::Bind { comp, cont } => {
                    env = env.clone().extend_clos(MValue::Var(999).into(), env);
                    cont
                },
                MComputation::Equate { lhs, rhs, body } => {
                    match unify(&lhs, &rhs, &env, &mut lenv) {
                        Err(UnifyError::Fail) => { return true },
                        Err(UnifyError::Occurs) => { return true },
                        Err(UnifyError::Open) => { return false }
                        _ => { return false }
                    }
                },
                MComputation::Lambda { body } => {
                    env = env.clone().extend_clos(MValue::Var(9999).into(), env);
                    body
                },
                MComputation::Exists { ptype, body } => {
                    let ident = lenv.fresh(ptype.clone());
                    env = env.clone().extend_lvar(ident);
                    body
                },
                MComputation::Rec { body } => {
                    env = env.clone().extend_clos(MValue::Var(9999).into(), env);
                    body
                },
                MComputation::App { op, arg } => op,
                _ => return false
            }
        }
    }
}