pub mod mterms;
mod lvar;
mod env;
mod step;
pub mod translate;
use std::rc::Rc;
use env::Env;
use im::vector::Vector;
use lvar::LogicVar;
use mterms::{MComputation, MValue};
use step::{close_val, empty_stack, step, Machine};
pub trait DeepClone {
    fn deep_clone(&self) -> Rc<Self>;
}

#[derive(Clone, Debug)]
pub enum VClosure {
    Clos { val : Rc<MValue>, env : Rc<Env> },
    LogicVar { lvar : Rc<LogicVar> } // by making this Rc we ensure that cloning is never deep
}

impl VClosure {
    fn val(&self) -> String { 
        match self {
            VClosure::Clos { val, env } => val.to_string(),
            VClosure::LogicVar { lvar } => "logic var".to_string(),
        }
    }
    
    fn deep_clone(self: &Rc<Self>) -> Rc<Self> {
        if self.has_unresolved_lvars() {
            match &**self {
                Self::Clos { val, env } => Self::Clos { val: val.clone(), env: env.deep_clone() },
                Self::LogicVar { lvar } => Self::LogicVar { lvar: Rc::new((**lvar).clone()) },
            }.into()
        }
        else {
            self.clone()
        }
    }
    
    pub fn has_unresolved_lvars(&self) -> bool {
        match self {
            Self::Clos { .. } => false,
            Self::LogicVar { lvar } => !lvar.resolved()
        }
    }
}


pub fn eval(comp : MComputation, env : Rc<Env>, mut fuel : usize) -> Vec<MValue> {

    let m = Machine { comp: comp.into() , env: env.clone(), stack: empty_stack().into(), done: false };
    println!("[DEBUG] initial env: ");
    let mut machines = vec![m];
    let mut values = vec![];
    
    while fuel > 0 && !machines.is_empty() {
        let (mut done, ms) : (Vec<Machine>, Vec<Machine>) = machines.into_iter().flat_map(|m| step(m)).partition(|m| m.done);
        println!("[DEBUG] machines: ");
        ms.iter().for_each(|m| {
            println!("[DEBUG]   comp: {}", m.comp);
            println!("[DEBUG]   stack size: {:?}", m.stack.len());
            println!("[DEBUG]   env size: {:?}", m.env.size())
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