use std::{collections::{HashMap, VecDeque}, rc::Rc};

use crate::{cbpv::terms::ValueType, eval::mterms::MVar};

use super::mterms::{MValue, MComputation};

#[derive(Clone)]
struct VClosure {
    val : Rc<MValue>,
    env : Rc<Env>
}
type Env = Vec<VClosure>;

fn extend_env(env : &Env, val : Rc<MValue>, venv : Rc<Env>) -> Rc<Env> {
    let mut newenv = env.clone();
    newenv.push(VClosure { val, env : venv });
    Rc::new(newenv)
}

fn lookup_env(env : &Env, i : usize) -> VClosure {
    env[i].clone()
}

fn close_fo_value(val : &Rc<MValue>, env : &Env, lenv : &LogicEnv) -> Rc<MValue> {
    match &**val {
        MValue::Var(MVar::Index(i)) => {
           let vclos = lookup_env(env, *i);
           close_fo_value(&vclos.val, &*vclos.env, lenv)
        },
        MValue::Var(MVar::Level(j)) => {
            match lookup_lenv(lenv, *j) {
                LogicVar::Closure(vclos) => close_fo_value(&vclos.val, &*vclos.env, lenv),
                LogicVar::Generator(ptype) => val.clone()
            }
        },
        MValue::Zero => val.clone(),
        MValue::Succ(v) => Rc::new(MValue::Succ(close_fo_value(&v.clone(), env, lenv))),
        MValue::Bool(b) => val.clone(),
        MValue::Nil => val.clone(),
        MValue::Cons(v, w) => 
            Rc::new(MValue::Cons(close_fo_value(&v, env, lenv), close_fo_value(&w, env, lenv))),
        MValue::Thunk(t) => unreachable!("trying to close thunk"),
    }
}

#[derive(Clone)]
enum LogicVar {
    Generator(ValueType),
    Closure(VClosure)
}

type LogicEnv = Vec<LogicVar>;

fn lookup_lenv(lenv : &LogicEnv, j : usize) -> LogicVar {
    lenv[j].clone()
}

fn extend_lenv_gen(lenv : &LogicEnv, ptype : ValueType) -> Rc<LogicEnv> {
    let mut newenv = lenv.clone();
    newenv.push(LogicVar::Generator(ptype));
    Rc::new(newenv)
}

fn extend_lenv_clos(lenv : &LogicEnv, clos : VClosure) -> Rc<LogicEnv> {
    let mut newenv = lenv.clone();
    newenv.push(LogicVar::Closure(clos));
    Rc::new(newenv)
}

fn lenv_newvar(lenv : &LogicEnv) -> MVar {
    MVar::Level(lenv.len() + 1)
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
    lenv : Rc<LogicEnv>,
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
                        vec![Machine { comp: body.clone(), stack: tail.to_vec().into(), env : extend_env(&*m.env, val.clone(), m.env.clone()), ..m}]
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
            vec![Machine { lenv : extend_lenv_gen(&m.lenv, ptype.clone()), ..m}]
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
            let closed_num = close_fo_value(&num, &m.env, &m.lenv);
            match &*closed_num {
                MValue::Var(MVar::Index(i)) => unreachable!(), // should be closed
                MValue::Var(MVar::Level(j)) => {
                    return vec![
                        Machine { comp: zk.clone(), lenv: extend_lenv_clos(&m.lenv, VClosure { val : Rc::new(MValue::Zero), env : m.env.clone() }), ..m.clone()},
                        Machine { comp: sk.clone(), lenv: 
                            extend_lenv_clos(&*extend_lenv_gen(&m.lenv, ValueType::Nat), 
                                VClosure { val : Rc::new(MValue::Succ(Rc::new(MValue::Var(MVar::Level(*j))))), env : m.env.clone() }),
                            ..m.clone()}
                    ]
                },
                MValue::Zero => vec![Machine { comp: zk.clone(), ..m}],
                MValue::Succ(rc) => vec![Machine { comp: sk.clone(), ..m}],
                _ => panic!("Ifz on something non-numerical")
            }
        },
        _ => unreachable!()
    }
}

enum Constraint { VarEq { var : MVar, val : Rc<MValue>} }

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