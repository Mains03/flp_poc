pub mod mterms;
mod step;
pub mod translate;
use std::rc::Rc;
use im::vector::Vector;

use mterms::{MComputation, MValue};
use step::{close_val, empty_stack, step, LogicVar, Machine};

#[derive(Clone, Debug)]
pub enum VClosure {
    Clos { val : Rc<MValue>, env : Rc<Env> },
    LogicVar { lvar : Rc<LogicVar> }
}

impl VClosure {
    fn val(&self) -> String { 
        match self {
            VClosure::Clos { val, env } => val.to_string(),
            VClosure::LogicVar { lvar } => "logic var".to_string(),
        }
    }
    
    fn deep_clone(&self) -> Self {
        match self {
            Self::Clos { val, env } => Self::Clos { val: val.clone(), env: env.deep_clone() },
            Self::LogicVar { lvar } => Self::LogicVar { lvar: Rc::new((**lvar).clone()) },
        }
    }
}

pub struct Env {
    env : Vector<VClosure>
}

impl Env {
    pub fn empty_env() -> Env { 
        Env { env : Vector::new() }
    }

    fn lookup_env(&self, i : usize) -> &VClosure {
        self.env.get(self.env.len() - i - 1).expect("indexing error")
    }
    
    pub fn extend_env(&self, vclos : VClosure) -> Env {
        let vector = self.env.push_back(vclos)
        Env { env : self.env.push_front(vclos) }
    }

    fn extend_env_clos(&self, val : Rc<MValue>, venv : Rc<Env>) -> Rc<Env> {
        self.extend_env( VClosure::Clos { val, env : venv })
    }

    fn extend_env_lvar(&self, lvar : Rc<LogicVar>) -> Rc<Env> {
        self.extend_env(VClosure::LogicVar { lvar })
    }

    pub fn deep_clone(env : Rc<Env>) -> Rc<Env> {
        let new_env : Env = env.iter().map(|vclos| vclos.deep_clone()).collect();
        new_env.into()
    }

}

pub fn eval(comp : MComputation, env : Rc<Env>, mut fuel : usize) -> Vec<MValue> {

    let m = Machine { comp: comp.into() , env: env.clone(), stack: empty_stack().into(), done: false };
    println!("[DEBUG] initial env: ");
    env.iter().for_each(|vclos| println!("[DEBUG]   {:?}: ", vclos));
    let mut machines = vec![m];
    let mut values = vec![];
    
    while fuel > 0 && !machines.is_empty() {
        let (mut done, ms) : (Vec<Machine>, Vec<Machine>) = machines.into_iter().flat_map(|m| step(m)).partition(|m| m.done);
        println!("[DEBUG] machines: ");
        ms.iter().for_each(|m| {
            println!("[DEBUG]   comp: {}", m.comp);
            println!("[DEBUG]   stack size: {:?}", m.stack.len());
            println!("[DEBUG]   env size: {:?}", m.env.len())
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