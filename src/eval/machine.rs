use std::{collections::{HashMap, VecDeque}, rc::Rc};

use crate::cbpv::terms::{Computation, Value, ValueType};

enum Env {
    Empty,
    Cons { 
        var : String, 
        clos : Rc<Closure>, // THIS SHOULD ONLY BE A VALUE CLOSURE - PLEASE USE EXTEND METHOD TO MAKE ENVS
        next : Option<Rc<Env>> }
}

impl Env {
    fn extend(self : Rc<Self>, var : String, val : Rc<Value>, env : Rc<Env>, gens : Generators) -> Rc<Self> {
        Rc::new(Env::Cons { var, clos : Rc::new(Closure { frame : Rc::new(Frame::Value(val)), gens, env}), next: Some(self)})
    }
    fn extendref(self : &Rc<Self>, var : &String, val : &Rc<Value>, env : &Rc<Env>, gens : &Generators) -> Rc<Self> {
        self.clone().extend(var.to_string(), val.clone(), env.clone(), gens.clone())
    }
    fn find(self : Rc<Env>, s : &String) -> Option<(Rc<Value>, Rc<Env>)> {
        let mut cenv = &*self;
        loop {
            match cenv {
                Env::Empty => { return None },
                Env::Cons { var, clos, next } => {
                    if *var == *s {
                        let Closure { frame, env, gens } = &**clos;
                        if let Frame::Value(val) = &**frame {
                            return Some((val.clone(), env.clone()))
                        } else { panic!("non-value closure found in env") }
                    }
                    let Closure { frame, env, gens } = &**clos;
                    if let Frame::Value(val) = &**frame {
                        if *var == *s { return Some((val.clone(), env.clone())) } else { 
                            if let Some(env) = next { cenv = env; continue; } else { return None }
                        }
                    }
                    return None
                }
            }
        }
    }
}

type Generators = HashMap<String, ValueType>;

pub enum Frame {
    Value(Rc<Value>),
    To(String, Rc<Computation>)
}

#[derive(Clone)]
pub struct Closure {
    frame: Rc<Frame>,
    env: Rc<Env>,
    gens: Generators 
}

type Stack = Vec<Closure>;

fn push_clos(stk : &Stack, frame : Frame, env : &Rc<Env>, gens : &Generators) -> Stack {
  let mut stk = stk.clone();
  stk.push(Closure { frame: Rc::new(frame), env : env.clone() , gens : gens.clone() });
  stk
}

#[derive(Clone)]
pub struct Machine {
    comp : Rc<Computation>,
    env  : Rc<Env>,
    gens : HashMap<String, ValueType>,
    stack : Stack,
    done : bool
}

pub fn step(m : Machine) -> Vec<Machine> {
    match &*(m.comp) {
        Computation::Return(val) => {
            match &*(m.stack).as_slice() {
                [] => vec![Machine { done: true, ..m }],
                [tail @ .., clos] => {
                    let Closure { frame , env, gens } = &*clos;
                    if let Frame::To(var, cont) = &**frame {
                        vec![Machine { comp: cont.clone(), stack : tail.to_vec(), env: env.extendref(var, val, &m.env, &m.gens), gens : gens.clone(), ..m }]
                    } else { panic!("return but no to frame in the stack") }
                },
                  _ => unreachable!()
              }
        },
        Computation::Bind { var, comp, cont } => 
            vec![Machine { comp: comp.clone(), stack: push_clos(&m.stack, Frame::To(var.to_string(), cont.clone()), &m.env, &m.gens), ..m}],
        Computation::Force(th) => todo!(),
        Computation::Lambda { var, body } => {
            match &*(m.stack).as_slice() {
                [] => panic!(),
                [tail @ .., clos] => {
                    let Closure { frame , env, gens} = &*clos;
                    if let Frame::Value(val) = &**frame {
                        vec![Machine { comp: body.clone(), stack: tail.to_vec(), env : m.env.extendref(var, val, env, gens), ..m}]
                    } else { panic!("lambda but no value frame in the stack") }
                },
                _ => unreachable!()
              }
        },
        Computation::App { op, arg } => 
            vec![Machine { comp: op.clone(), stack: push_clos(&m.stack, Frame::Value(arg.clone()), &m.env, &m.gens), ..m}]
        ,
        Computation::Choice(choices) => 
          choices.iter().map(|c| Machine { comp: c.clone(), ..m.clone()}).collect(),
        Computation::Exists { var, ptype, body } => {
            let mut gens = m.gens.clone();
            gens.insert(var.clone(), ptype.clone());
            vec![Machine { gens : gens, ..m}]
        }
        Computation::Equate { lhs, rhs, body } => {
          let constraints = unify(lhs, rhs);
          if constraints.is_empty() {
            vec![]
          }
          else {
            let old_env = m.env.clone();
            let new_env = constraints.iter().fold(m.env, 
                |env, Constraint::VarEq{ var, val}| env.extendref(var, val, &env, &m.gens));
            vec![Machine { comp: body.clone(), env: new_env, ..m}]
          }
        },
        Computation::Ifz { num, zk, sk } => {
            match &**num {
                Value::Var(var ) => {
                    let mut x = var;
                    if let Some(ptype) = m.gens.get(var) {
                        if *ptype == ValueType::Nat {
                            let newvar = var.to_string() + "1";
                            let mut newgens = m.gens.clone();
                            newgens.insert(newvar.clone(), ValueType::Nat);
                            return vec![
                                Machine { comp: zk.clone(), env: m.env.extendref(var, &Rc::new(Value::Zero), &m.env, &m.gens), ..m.clone()},
                                Machine { comp: sk.clone(), env: m.env.extendref(var, &Rc::new(Value::Succ(Rc::new(Value::Var(newvar)))), &m.env, &newgens), 
                                    gens : newgens, ..m.clone()}
                            ]
                        }
                        else { panic!("performing a zero test on a non-nat generator")}
                    }
                    if let Some((val, env)) = m.env.find(var) {
                    }
                    todo!()
                },
                Value::Zero => vec![Machine { comp: zk.clone(), ..m}],
                Value::Succ(rc) => vec![Machine { comp: sk.clone(), ..m}],
                _ => panic!("Ifz on something non-numerical")
            }
        },
        _ => unreachable!()
    }
}

enum Constraint { VarEq { var : String, val : Rc<Value>} }

fn unify(lhs : &Rc<Value>, rhs : &Rc<Value>) -> Vec<Constraint> {
    let mut out: Vec<Constraint> = vec![];
    let mut q : VecDeque<(&Rc<Value>, &Rc<Value>)> = VecDeque::new();
    q.push_back((lhs, rhs));
    while let Some((lhs, rhs)) = q.pop_front() {
        match (&**lhs, &**rhs) {
            (Value::Var(x), v) => { 
                if (*v).occurs(x) { out = vec![]; break; }
                out.push(Constraint::VarEq {var : x.to_string(), val : rhs.clone()})
            },
            (v , Value::Var(x)) => {
                if (*v).occurs(x) { out = vec![]; break; }
                out.push(Constraint::VarEq {var : x.to_string(), val : lhs.clone()})
            },
            (Value::Zero, Value::Zero) => continue,
            (Value::Zero, _) => { out = vec![]; break },
            (Value::Succ(v), Value::Succ(w)) => q.push_back((v, w)),
            (Value::Succ(_), _) => { out = vec![]; break }
            (Value::Nil, Value::Nil) => continue,
            (Value::Nil, _) => {out = vec![]; break },
            (Value::Cons(x, xs), Value::Cons(y, ys)) => { q.push_back((x, y)); q.push_back((xs, ys)) }
            (Value::Cons(_, _), _) => { out = vec![]; break }
            _ => continue
        }
    }
    return out
} 