use std::rc::Rc;

use crate::{cbpv::terms::{Computation, Value}, parser::syntax::r#type::Type};

pub enum Env {
    Empty,
    Cons { var : String, val : Rc<Value>, env : Rc<Env>, next : Option<Rc<Env>> }
}

impl Env {
    fn extend(self : Rc<Self>, var : String, val : Rc<Value>, env : Rc<Env>) -> Rc<Self> {
        Rc::new(Env::Cons { var, val, env, next: Some(self)})
    }
    fn find(self : Rc<Env>, s : String) -> Option<Rc<Value>> {
        let mut cenv = &*self;
        loop {
            match cenv {
                Env::Empty => { return None },
                Env::Cons { var, val, env, next } => {
                    if *var == s { return Some(val.clone()) } else { 
                        if let Some(env) = next { cenv = env; continue; } else { return None }
                    }
                }
            }
        }
    }
}
pub enum Frame {
    Value(Rc<Value>),
    To(String, Rc<Computation>)
}

#[derive(Clone)]
pub struct Closure {
    frame: Rc<Frame>,
    env: Rc<Env>
}

type Stack = Vec<Closure>;

fn push_clos(stk : &Stack, frame : Frame, env : Rc<Env>) -> Stack {
  let mut stk = stk.clone();
  stk.push(Closure { frame: Rc::new(frame), env });
  stk
}

#[derive(Clone)]
pub struct Machine {
    comp : Rc<Computation>,
    env  : Rc<Env>,
    gens : Vec<(String, Type)>,
    stack : Stack,
    done : bool
}

pub fn step<V: Eq>(m : Machine) -> Vec<Machine> {
    match &*(m.comp) {
        Computation::Return(val) => {
            match &*(m.stack).as_slice() {
                [] => vec![Machine { done: true, ..m }],
                [tail @ .., clos] => {
                    let Closure { frame , env } = &*clos;
                    if let Frame::To(var, cont) = &**frame {
                        vec![Machine { comp: cont.clone(), stack : tail.to_vec(), env: env.clone().extend(var.clone(), val.clone(), m.env), ..m }]
                    } else { panic!() }
                },
                  _ => unreachable!()
              }
        },
        Computation::Bind { var, comp, cont } => 
            vec![Machine { comp: comp.clone(), stack: push_clos(&m.stack, Frame::To(var.to_string(), cont.clone()), m.env.clone()), ..m}],
        Computation::Force(th) => todo!(),
        Computation::Lambda { var, body } => {
            match &*(m.stack).as_slice() {
                [] => panic!(),
                [tail @ .., clos] => {
                    let Closure { frame , env } = &*clos;
                    if let Frame::Value(val) = &**frame {
                        vec![Machine { comp: body.clone(), stack: tail.to_vec(), env : m.env.clone().extend(var.clone(), val.clone(), env.clone()), ..m}]
                    } else { panic!() }
                },
                  _ => unreachable!()
              }
        },
        Computation::App { op, arg } => 
            vec![Machine { comp: op.clone(), stack: push_clos(&m.stack, Frame::Value(arg.clone()), m.env.clone()), ..m}]
        ,
        Computation::Choice(choices) => 
          choices.iter().map(|c| Machine { comp: c.clone(), ..m.clone()}).collect(),
        Computation::Exists { var, ptype, body } => {
            let mut gens = m.gens.clone();
            gens.push((var.clone(), ptype.clone()));
            vec![Machine { gens : gens, ..m}]
        }
        Computation::Equate { lhs, rhs, body } => todo!(),
        Computation::PM(pm) => todo!(),
        _ => unreachable!()
    }
}