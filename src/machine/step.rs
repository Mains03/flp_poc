use std::rc::Rc;
use crate::machine::{lvar, senv, senv::SuspAt, value_type::ValueType};
use super::{lvar::LogicEnv, mterms::{MComputation, MValue}, senv::SuspEnv, unify::UnifyError, Env, Ident, VClosure};
use crate::machine::unify::unify;
    
#[derive(Clone, Debug)]
enum Frame {
    Value(Rc<MValue>),
    To(Rc<MComputation>),
    Set(Ident, Rc<MComputation>),
}

#[derive(Clone, Debug)]
pub struct StkClosure {
    frame: Frame,
    env: Rc<Env>,
}

#[derive(Clone, Debug)]
pub enum Stack {
    Nil,
    Cons(StkClosure, Rc<Stack>),
}

impl Stack {
    pub fn empty_stack() -> Rc<Stack> { Rc::new(Stack::Nil) }

    fn push_closure(self: &Rc<Stack>, frame: Frame, env: Rc<Env>) -> Rc<Stack> {
        Stack::Cons(StkClosure { frame, env }, self.clone()).into()
    }

    fn push_susp(self: &Rc<Stack>, ident: Ident, c: Rc<MComputation>, env: Rc<Env>) -> Rc<Stack> {
        Stack::push_closure(self, Frame::Set(ident, c), env)
    }
}

#[derive(Clone)]
pub struct Machine {
    pub comp : Rc<MComputation>,
    pub stack: Rc<Stack>,
    pub env  : Rc<Env>,
    pub lenv : LogicEnv,
    pub senv : SuspEnv,
    pub done : bool
}

impl Machine {

    fn eval_susp_then(self, a : SuspAt) -> Machine {
        Machine { comp : a.comp, env : a.env, stack : self.stack.push_susp(a.ident, self.comp, self.env), ..self  }
    }

    pub fn step(self) -> Vec<Machine> {
        let m = self;
        
        match &*m.comp {

            MComputation::Return(val) => {
                match &*m.stack {
                    Stack::Nil => {
                        match m.senv.next() {
                            Some(a) =>
                                vec![ m.eval_susp_then(a) ],
                                // vec![Machine { comp: c.clone(), env: env.clone(), stack: m.stack.push_susp(*ident, m.comp, m.env), ..m }],
                            None => vec![Machine { done: true, ..m }],
                        }
                    }
                    Stack::Cons(clos, tail) => {
                        let StkClosure { frame, env } = clos.clone();
                        match frame {
                            Frame::Value(_) => unreachable!("return throws value to a value"),
                            Frame::To(cont) => {
                                let new_env = env.extend_val(val.clone(), m.env.clone());
                                vec![Machine { comp: cont.clone(), stack: tail.clone(), env: new_env, ..m }]
                            }
                            Frame::Set(i, cont) => {
                                let mut senv = m.senv;
                                senv.set(&i, val.clone(), m.env);
                                vec![Machine { comp: cont.clone(), stack: tail.clone(), env: env.clone(), senv, ..m }]
                            }
                        }
                    }
                }
            },

            MComputation::Bind { comp, cont } => {
                match &**comp {
                    MComputation::Return(v) => {
                        let env = m.env.extend_val(v.clone(), m.env.clone());
                        vec![Machine { comp : cont.clone(), env, ..m }]
                    },
                    _ => {
                        let mut senv = m.senv;
                        let env = &m.env;
                        let ident = senv.fresh(&comp, &m.env);
                        let env = env.extend_susp(ident);
                        vec![Machine { comp : cont.clone(), env, senv : senv, ..m}]
                    }
                }
            },
            MComputation::Force(v) => {
                let vclos = VClosure::Clos { val: v.clone(), env: m.env.clone() };
                match vclos.close_head(&m.lenv, &m.senv) {
                    Ok(vclos) => 
                        match vclos {
                            VClosure::Clos { val, env } => {
                                match &*val {
                                    MValue::Thunk(t) => vec![Machine { comp : t.clone(), env : env.clone(), ..m}],
                                _ => panic!("shouldn't be forcing a non-thunk value")
                                } 
                            },
                            VClosure::LogicVar { ident } => panic!("shouldn't be forcing a logic variable"),
                            VClosure::Susp { ident } => unreachable!("shouldn't be forcing a suspension"),
                        }
                    Err(a) => {
                        vec![Machine { comp : a.comp, env : a.env, stack : m.stack.push_susp(a.ident, m.comp, m.env), ..m  }]
                    },
                }
            },

            MComputation::Lambda { body } => {
                match &*m.stack {
                    Stack::Cons(StkClosure { frame, env }, tail) => {
                        if let Frame::Value(val) = frame {
                            let env = m.env.extend_val(val.clone(), env.clone());
                            vec![Machine { comp: body.clone(), stack: tail.clone(), env, ..m }]
                        } else {
                            panic!("lambda but no value frame in the stack")
                        }
                    },
                    Stack::Nil => panic!("lambda met with empty stack")
                }
            },

            MComputation::App { op, arg } =>
                vec![Machine { comp: op.clone(), stack: m.stack.push_closure(Frame::Value(arg.clone()), m.env.clone()), ..m }],

            MComputation::Choice(choices) => 
              choices.iter().map(|c| Machine { comp: c.clone(), ..m.clone()}).collect(),

            MComputation::Exists { ptype, body } => {
                let mut lenv = m.lenv;
                let ident = lenv.fresh(ptype.clone());
                vec![Machine { comp : body.clone(), env : m.env.extend_lvar(ident), lenv : lenv, ..m}]
            }

            MComputation::Equate { lhs, rhs, body } => {
                let mut lenv = m.lenv;
                match unify(&lhs, &rhs, &m.env, &mut lenv, &m.senv) {
                    Ok(()) => vec![Machine { comp : body.clone(), lenv : lenv, ..m }],
                    Err(UnifyError::Susp(a)) => {
                        vec![Machine { comp : a.comp, env : a.env, stack : m.stack.push_susp(a.ident, m.comp, m.env), lenv : lenv, ..m  }]
                    }
                    Err(_) => vec![]
                }
            },

            MComputation::Ifz { num, zk, sk } => {
                let vclos = VClosure::mk_clos(num, &m.env);
                match vclos.close_head(&m.lenv, &m.senv) {
                    Err(a) =>
                        vec![Machine { comp : a.comp, env : a.env, stack : m.stack.push_susp(a.ident, m.comp, m.env), ..m  }],
                    Ok(VClosure::Clos { val, env }) => {
                        match &*val {
                            MValue::Zero => vec![Machine { comp: zk.clone(), ..m}],
                            MValue::Succ(v) => {
                                let env = m.env.extend_val(v.clone(), env.clone());
                                vec![Machine { comp: sk.clone(), env, ..m}]
                            }
                            _ => panic!("Ifz on something non-numerical")
                        }
                    },
                    Ok(VClosure::LogicVar { ident }) => { // must be unresolved, by structure of close_head
                        let m_zero  = {
                            let mut lenv = m.lenv.clone(); // make a new logic env
                            lenv.set_vclos(ident, VClosure::Clos { val: MValue::Zero.into(), env: Env::empty()});

                            Machine { comp: zk.clone(), lenv : lenv, ..m.clone()}
                        };
                        
                        let m_succ = {
                            let mut lenv = m.lenv.clone();
                            let ident_lvar_succ = lenv.fresh(ValueType::Nat);
                            
                            lenv.set_vclos(ident, VClosure::Clos { 
                                val : MValue::Succ(Rc::new(MValue::Var(0))).into(), 
                                env : Env::empty().extend_lvar(ident_lvar_succ)
                            }.into());
                            
                            let new_env = m.env.extend_lvar(ident_lvar_succ);

                            Machine { comp: sk.clone(), lenv : lenv, env : new_env, ..m.clone()}
                        };

                        vec![m_zero, m_succ]
                    },
                    Ok(VClosure::Susp { ident }) => unreachable!("shouldn't")
                }
            },

            MComputation::Match { list, nilk, consk } => {
                let vclos = VClosure::Clos { val : list.clone(), env: m.env.clone() };
                let closed_list = vclos.close_head(&m.lenv, &m.senv);
                match closed_list {
                    Err(a) => vec![Machine { comp : a.comp, env : a.env, stack : m.stack.push_susp(a.ident, m.comp, m.env), ..m  }],
                    Ok(vclos) => 
                        match vclos {
                            VClosure::Clos { val, env } => {
                                match &*val {
                                    MValue::Nil => vec![Machine { comp: nilk.clone(), ..m}],
                                    MValue::Cons(v, w) => {
                                        let new_menv = 
                                            m.env.extend_val(v.clone(), env.clone()).extend_val(w.clone(), env.clone());
                                        vec![Machine { comp: consk.clone(), env : new_menv, ..m}]
                                    },
                                    _ => panic!("Match on non-list")
                                }
                            },
                            VClosure::LogicVar { ident } => {  // must be unresolved, by structure of close_head
                                                              
                                let ptype = match m.lenv.get_type(ident) {
                                    ValueType::List(t) => t,
                                    _ => panic!("matching on a non-list logical variable")
                                };

                                let m_nil  = {
                                    
                                    let mut lenv = m.lenv.clone();
                                    lenv.set_vclos(ident, VClosure::Clos { val: MValue::Nil.into(), env: Env::empty() });

                                    Machine { comp: nilk.clone(), lenv : lenv, ..m.clone()}
                                };
                                
                                let m_cons = {
                                    
                                    let mut lenv = m.lenv.clone();
                                    let head_ident = lenv.fresh(*ptype.clone());
                                    let tail_ident = lenv.fresh(ValueType::List(ptype));
                                    
                                    lenv.set_vclos(ident, VClosure::Clos {
                                        val: MValue::Cons(Rc::new(MValue::Var(1)), Rc::new(MValue::Var(0))).into(),
                                        env: Env::empty().extend_lvar(head_ident).extend_lvar(tail_ident)
                                    });
                                    
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
                    Err(a) => vec![Machine { comp : a.comp, env : a.env, stack : m.stack.push_susp(a.ident, m.comp, m.env), ..m  }],
                    Ok(vclos) => 
                        match vclos {
                            VClosure::Clos { val, env } => {
                                match &*val {
                                    MValue::Inl(v) => {
                                        let old_env = env.clone();
                                        let new_env = m.env.extend_val(v.clone(), old_env.clone());
                                        vec![Machine { comp: inlk.clone(), env : new_env, ..m}]
                                    },
                                    MValue::Inr(v) => {
                                        let old_env = env.clone();
                                        let new_env = m.env.extend_val(v.clone(), old_env.clone());
                                        vec![Machine { comp: inrk.clone(), env : new_env, ..m}]
                                    },
                                    _ => panic!("Match on non-list")
                                }
                            },
                            VClosure::LogicVar { ident } => {  // must be unresolved, by structure of close_head
                                                              
                                let (ptype1, ptype2) = match m.lenv.get_type(ident) {
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
                let env = m.env.extend_val(m.comp.thunk(), m.env.clone());
                vec![Machine { comp : body.clone(), env, ..m }] 
            },
        }
    }
    
}