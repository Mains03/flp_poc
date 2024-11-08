pub mod mterms;
mod step;
pub mod translate;
use std::rc::Rc;

use mterms::{MComputation, MValue};
use step::{close_val, empty_stack, step, LogicVar, Machine};

#[derive(Clone, Debug)]
pub enum VClosure {
    Clos { val : Rc<MValue>, env : Rc<Env> },
    LogicVar { lvar : Rc<LogicVar> }
}

pub type Env = Vec<VClosure>;

pub fn empty_env() -> Rc<Env> { Rc::new(vec![]) }

pub fn extend_env(env : &Env, vclos : VClosure) -> Rc<Env> {
    let mut newenv = env.clone();
    newenv.push(vclos);
    Rc::new(newenv)
}

fn extend_env_clos(env : &Env, val : Rc<MValue>, venv : Rc<Env>) -> Rc<Env> {
    extend_env(env, VClosure::Clos { val, env : venv })
}

fn lookup_env(env : &Env, i : usize) -> VClosure {
    env.get(env.len() - i - 1).expect(&("indexing ".to_owned() + &i.to_string() + " in env of length " + &env.len().to_string())).clone()
}

pub fn eval(comp : MComputation, env : Env, mut fuel : usize) -> Vec<MValue> {

    let m = Machine { comp: comp.into() , env: env.clone().into(), stack: empty_stack().into(), done: false };
    println!("[DEBUG] initial env: ");
    env.iter().for_each(|vclos| println!("[DEBUG]   {:?}: ", vclos));
    let mut machines = vec![m];
    let mut values = vec![];
    
    while fuel > 0 && !machines.is_empty() {
        let (mut done, ms) : (Vec<Machine>, Vec<Machine>) = machines.into_iter().flat_map(|m| step(m)).partition(|m| m.done);
        println!("[DEBUG] machines: ");
        ms.iter().for_each(|m| {
            println!("[DEBUG]   comp: {:?}", m.comp);
            //println!("[DEBUG]   stack: {:?}", m.stack);
            //println!("[DEBUG]   env: {:?}", m.env)
        });
        values.append(&mut done);
        machines = ms;
        fuel -= 1
    }
    
    values.iter().map(|m| {
        match &*m.comp {
            MComputation::Return(v) => close_val(&VClosure::Clos { val: v.clone(), env: m.env.clone() }),
            _ => unreachable!()
        }
    }).collect()
}