pub mod mterms;
mod vclosure;
mod lvar;
mod env;
mod unify;
mod step;
pub mod translate;
use std::rc::Rc;
use env::Env;
use im::vector::Vector;
use lvar::LogicVar;
use mterms::{MComputation, MValue};
use step::{empty_stack, step, Machine};
use vclosure::VClosure;

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
            MComputation::Return(v) => VClosure::Clos { val: v.clone(), env: m.env.clone() }.close_val(),
            _ => unreachable!()
        }
    }).collect()
}