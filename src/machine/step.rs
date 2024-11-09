use std::{borrow::Borrow, cell::RefCell, collections::{HashMap, VecDeque}, ptr, rc::Rc};
use crate::cbpv::terms::ValueType;
use super::{lvar::LogicVar, mterms::{MComputation, MValue}, Env, VClosure};

    
impl VClosure {
    fn occurs_lvar(&self, lvar : &LogicVar) -> bool {
        let vclos = close_head(&self);
        match &*vclos {
            VClosure::Clos { val, env } => {
                match &**val {
                    MValue::Succ(v) => VClosure::Clos {val : v.clone(), env: env.clone() }.occurs_lvar(lvar),
                    MValue::Cons(v, w) => 
                        VClosure::Clos { val : v.clone(), env: env.clone()}.occurs_lvar(lvar)
                        || VClosure::Clos { val : w.clone(), env : env.clone()}.occurs_lvar(lvar),
                    MValue::Var(_) => unreachable!("value should be head-closed in occurs check"),
                    MValue::Thunk(_) => panic!("mustn't be doing occurs to a computation"),
                    _ => false
                }
            },
            VClosure::LogicVar { lvar: _lvar } => *lvar == **_lvar,
        }
    }
}

fn close_head(vclos : &VClosure) -> Rc<VClosure> {
    let mut vclos = vclos.clone();
    loop {
        vclos = match vclos {
            VClosure::Clos { ref val, ref env } => {
                match **val {
                    MValue::Var(i) => env.lookup(i).clone(),
                    _ => break
                }
            },
            VClosure::LogicVar { ref lvar } => {
                match lvar.get() {
                    Some(v) => v,
                    None => break,
                }
            }
        }
    }
    Rc::new(vclos)
}

pub fn close_val(vclos : &VClosure) -> MValue {
    match vclos {
        VClosure::Clos { val,  env } => {
            // println!("[DEBUG] CLOSING {} in env of size {}: {:#?}", val, env.len(), *env);
            match &**val {
                MValue::Var(i) => close_val(env.lookup(*i)),
                MValue::Zero => MValue::Zero,
                MValue::Succ(v) => MValue::Succ(close_val(&VClosure::Clos { val: v.clone(), env : env.clone() }).into()),
                MValue::Nil => MValue::Nil,
                MValue::Cons(v, w) => 
                    MValue::Cons(
                        close_val(&VClosure::Clos { val: v.clone(), env : env.clone() }).into(),
                        close_val(&VClosure::Clos { val: w.clone(), env : env.clone() }).into()
                    ),
                MValue::Pair(fst, snd) => 
                    MValue::Pair(
                        close_val(&VClosure::Clos{ val : fst.clone(), env : env.clone ()}).into(),
                        close_val(&VClosure::Clos{ val : snd.clone(), env : env.clone ()}).into(),
                    ),
                MValue::Inl(v) =>
                    MValue::Inl(close_val(&VClosure::Clos{ val : v.clone(), env : env.clone() }).into()),
                MValue::Inr(v) => 
                    MValue::Inr(close_val(&VClosure::Clos{ val : v.clone(), env : env.clone() }).into()),
                MValue::Thunk(t) => panic!("shouldn't be returning a thunk anyway: {}", *t),
            }
        },
        VClosure::LogicVar { ref lvar } => {
            match lvar.get() {
                Some(v) => close_val(&v),
                None => panic!("unresolved logic var"),
            }
        }
    }
}

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
            let w = close_head(&VClosure::Clos { val: v.clone(), env: m.env.clone() });
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
            let closed_num = close_head(&vclos);
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
                        let closed_num_zero = close_head(&vclos_zero);
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
                        let closed_num_succ = close_head(&vclos_succ);
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
            let closed_list = close_head(&vclos);
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
                        let closed_list_nil = close_head(&vclos_nil);
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
                        let closed_num_cons = close_head(&vclos_cons);
                        if let VClosure::LogicVar { lvar } = &*closed_num_cons {
                            lvar.set_val(MValue::Cons(Rc::new(MValue::Var(1)), Rc::new(MValue::Var(0))).into(), lvar_env)
                        }
                        else { unreachable!("closure was returned when closure shouldn't be returned: {:?}", close_head(&vclos_cons).val() ) } 
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
            let closed_sum = close_head(&vclos);
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
                        let closed = close_head(&vclos); // re-find lvar in deep clone
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
                        let closed = close_head(&vclos); // re-find lvar in deep clone
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

fn unify(lhs : &Rc<MValue>, rhs : &Rc<MValue>, env : &Rc<Env>) -> bool { 
    let mut q : VecDeque<(Rc<VClosure>, Rc<VClosure>)> = VecDeque::new();
    
    let lhs_clos = VClosure::Clos { val: lhs.clone(), env: env.clone() }.into();
    let rhs_clos = VClosure::Clos { val: rhs.clone(), env: env.clone() }.into();
    
    q.push_back((lhs_clos, rhs_clos));
    while let Some((lhs, rhs)) = q.pop_front() {
        let lhs = close_head(&lhs);
        let rhs = close_head(&rhs);
        match (&*lhs, &*rhs) {
            (VClosure::LogicVar { lvar }, _) => { 
                // the head of the LHS has been closed, so it must be a free logic variable
                if rhs.occurs_lvar(lvar) { return false }
                lvar.set_vclos(&rhs);
            },
            (_, VClosure::LogicVar { lvar }) => { 
                if lhs.occurs_lvar(lvar) { return false }
                lvar.set_vclos(&lhs);
            },
            (VClosure::Clos { val : lhs_val, env: lhs_env}, VClosure::Clos { val : rhs_val, env : rhs_env }) =>
                match (&**lhs_val, &**rhs_val) {
                    (MValue::Zero, MValue::Zero) => { continue },
                    (MValue::Zero, _) => { return false },
                    (MValue::Succ(v), MValue::Succ(w)) => {
                        q.push_back((VClosure::Clos { val: v.clone(), env: lhs_env.clone() }.into(), VClosure::Clos { val : w.clone(), env : rhs_env.clone()}.into()));
                    }
                    (MValue::Succ(_), _) => { return false }
                    (MValue::Nil, MValue::Nil) => continue,
                    (MValue::Nil, _) => { return false },
                    (MValue::Cons(x, xs), MValue::Cons(y, ys)) => { 
                        q.push_back((VClosure::Clos { val: x.clone(), env: lhs_env.clone() }.into(), VClosure::Clos { val : y.clone(), env : rhs_env.clone()}.into()));
                        q.push_back((VClosure::Clos { val: xs.clone(), env: lhs_env.clone() }.into(), VClosure::Clos { val : ys.clone(), env : rhs_env.clone()}.into()));
                    }
                    (MValue::Cons(_, _), _) => { return false }
                    _ => continue
                }
        }
    }
    return true
} 
