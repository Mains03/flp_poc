use std::{borrow::Borrow, cell::RefCell, collections::{HashMap, VecDeque}, rc::Rc};
use crate::{cbpv::terms::ValueType};
use super::mterms::{MValue, MComputation};

#[derive(Clone)]
struct LogicVar {
    ptype : ValueType,
    vclos : RefCell<Option<VClosure>>
}

impl LogicVar {
    fn new(ptype : &ValueType) -> Rc<Self> {
        Rc::new(LogicVar { 
            ptype: ptype.clone(), 
            vclos : RefCell::new(None)
        })
    }

    fn with_val_new(ptype : &ValueType, val : &Rc<MValue>, env : &Rc<Env>) -> Self {
        LogicVar {
            ptype: ptype.clone(), 
            vclos : RefCell::new(Some(VClosure::Clos { val: val.clone(), env : env.clone() }))
        }
    }
    
    fn clone(&self) -> Self  {
        LogicVar {
            ptype : self.ptype.clone(),
            vclos : RefCell::new(self.vclos.borrow().clone())
        }
    }
    
    fn set(&self, val : Rc<MValue>, env : Rc<Env>) {
        *(self.vclos.borrow_mut()) = Some(VClosure::Clos { val, env });
    }
}
    
#[derive(Clone)]
enum VClosure {
    Clos { val : Rc<MValue>, env : Rc<Env> },
    LogicVar { lvar : Rc<LogicVar> }
}
type Env = Vec<VClosure>;

fn empty_env() -> Rc<Env> { Rc::new(vec![]) }

fn extend_env(env : &Env, vclos : VClosure) -> Rc<Env> {
    let mut newenv = env.clone();
    newenv.push(vclos);
    Rc::new(newenv)
}

fn extend_env_clos(env : &Env, val : Rc<MValue>, venv : Rc<Env>) -> Rc<Env> {
    extend_env(env, VClosure::Clos { val, env : venv })
}

fn lookup_env(env : &Env, i : usize) -> VClosure {
    env[i].clone()
}

fn close_head(vclos : &VClosure) -> Rc<VClosure> {
    let mut vclos = vclos.clone();
    loop {
        vclos = match vclos {
            VClosure::Clos { ref val, ref env } => {
                match **val {
                    MValue::Var(i) => lookup_env(&env, i),
                    _ => break
                }
            },
            VClosure::LogicVar { ref lvar } => {
                match lvar.vclos.borrow().clone() {
                    Some(v) => v,
                    None => break,
                }
            }
        }
    }
    Rc::new(vclos)
}

enum Frame {
    Value(Rc<MValue>),
    To(Rc<MComputation>)
}

#[derive(Clone)]
struct Closure {
    frame : Rc<Frame>,
    env : Rc<Env>
}

type Stack = Vec<Closure>;

fn push_closure(stack : &Stack, frame : Frame, env : Rc<Env>) -> Rc<Stack> {
    let mut stk = stack.clone();
    stk.push(Closure { frame: frame.into(), env });
    Rc::new(stk)
}

#[derive(Clone)]
pub struct Machine {
    comp : Rc<MComputation>,
    env  : Rc<Env>,
    stack : Rc<Stack>,
    done : bool
}

pub fn step(m : Machine) -> Vec<Machine> {
    match &*(m.comp) {
        MComputation::Return(val) => {
            match &*(m.stack).as_slice() {
                [] => vec![Machine { done: true, ..m }],
                [tail @ .., clos] => {
                    let Closure { frame , env } = &*clos;
                    if let Frame::To(cont) = &**frame {
                        vec![Machine { comp: cont.clone(), stack : Rc::new(tail.to_vec()), ..m }]
                    } else { panic!("return but no to frame in the stack") }
                },
                  _ => unreachable!()
              }
        },
        MComputation::Bind { comp, cont } => 
            vec![Machine { comp: comp.clone(), stack: push_closure(&m.stack, Frame::To(cont.clone()), m.env.clone()), ..m}],
        MComputation::Force(th) => todo!(),
        MComputation::Lambda { body } => {
            match &*(m.stack).as_slice() {
                [] => panic!("lambda met with empty stack"),
                [tail @ .., clos] => {
                    let Closure { frame , env} = &*clos;
                    if let Frame::Value(val) = &**frame {
                        vec![Machine { comp: body.clone(), stack: tail.to_vec().into(), env : extend_env_clos(&*m.env, val.clone(), m.env.clone()), ..m}]
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
            vec![Machine { env : extend_env(&*m.env, VClosure::LogicVar { lvar: LogicVar::new(ptype) }), ..m}]
        }
        MComputation::Equate { lhs, rhs, body } => {
          let constraints = unify(lhs, rhs);
          if constraints.is_empty() {
            vec![]
          }
          else {
            todo!()
          }
        },
//            let old_env = m.env.clone();
//            let new_env = constraints.iter().fold(m.env, 
//                |env, Constraint::VarEq{ var, val}| extend_env(&env, val, &env));
//            vec![Machine { comp: body.clone(), env: new_env, ..m}]
//          }
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

                        let env_zero = Rc::new((*m.env).clone());
                        let vclos_zero = VClosure::Clos { val : num.clone(), env: env_zero.clone() };
                        let closed_num_zero = close_head(&vclos_zero);
                        if let VClosure::LogicVar { lvar } = &*closed_num_zero {
                            lvar.set(MValue::Zero.into(), empty_env())
                        }
                        else { unreachable!() } 

                        Machine { comp: zk.clone(), env : env_zero, ..m.clone()}
                    };
                    
                    let m_succ = {
                        let lvar_succ = LogicVar::new(&ValueType::Nat);
                        let mut lvar_env = empty_env();
                        lvar_env.push(VClosure::LogicVar { lvar: lvar_succ });

                        let env_succ = Rc::new((*m.env).clone());
                        let vclos_succ = VClosure::Clos { val : num.clone(), env: env_succ.clone() };
                        let closed_num_succ = close_head(&vclos_succ);
                        if let VClosure::LogicVar { lvar } = &*closed_num_succ {
                            lvar.set(MValue::Succ(Rc::new(MValue::Var(1))).into(), lvar_env)
                        }
                        else { unreachable!() } 

                        Machine { comp: sk.clone(), env : env_succ, ..m.clone()}
                    };

                    vec![m_zero, m_succ]
                }
            }
        },
        _ => unreachable!()
    }
}

enum Constraint { VarEq { var : LogicVar, val : Rc<MValue>} }

fn unify(lhs : &Rc<MValue>, rhs : &Rc<MValue>) -> Vec<Constraint> {
    let mut out: Vec<Constraint> = vec![];
    let mut q : VecDeque<(&Rc<MValue>, &Rc<MValue>)> = VecDeque::new();
    q.push_back((lhs, rhs));
    while let Some((lhs, rhs)) = q.pop_front() {
        match (&**lhs, &**rhs) {
            (MValue::Var(x), v) => { 
                if (*v).occurs(x) { out = vec![]; break; }
                out.push(Constraint::VarEq {var : x.clone(), val : rhs.clone()})
            },
            (v , MValue::Var(x)) => {
                if (*v).occurs(x) { out = vec![]; break; }
                out.push(Constraint::VarEq {var : x.clone(), val : lhs.clone()})
            },
            (MValue::Zero, MValue::Zero) => continue,
            (MValue::Zero, _) => { out = vec![]; break },
            (MValue::Succ(v), MValue::Succ(w)) => q.push_back((v, w)),
            (MValue::Succ(_), _) => { out = vec![]; break }
            (MValue::Nil, MValue::Nil) => continue,
            (MValue::Nil, _) => {out = vec![]; break },
            (MValue::Cons(x, xs), MValue::Cons(y, ys)) => { q.push_back((x, y)); q.push_back((xs, ys)) }
            (MValue::Cons(_, _), _) => { out = vec![]; break }
            _ => continue
        }
    }
    return out
} 