pub mod mterms;
mod vclosure;
mod env;
mod lvar;
mod senv;
mod unify;
mod step;
mod union_find;
mod value_type;
pub mod translate;
use std::{io::Write, rc::Rc};
use env::Env;
use im::vector::Vector;
use lvar::LogicEnv;
use mterms::{MComputation, MValue};
use senv::SuspEnv;
use step::{empty_stack, Machine};
use vclosure::VClosure;
use std::io::stdout;

pub type Ident = usize;
type ValueInEnv = (Rc<MValue>, Rc<Env>);
type ComputationInEnv = (Rc<MComputation>, Rc<Env>);


pub fn eval(comp : MComputation, env : Rc<Env>) {

    // println!("[DEBUG] main stmt: {}", comp.clone()) ;
    let m = Machine { comp: comp.into() , env: env.clone(), stack: empty_stack(), lenv : LogicEnv::new().into(), senv : SuspEnv::new().into(), done: false };
    let mut machines = vec![m];
    let mut solns = 0;
    while !machines.is_empty() {

        let (done, ms) : (Vec<Machine>, Vec<Machine>) = machines.into_iter()
            .flat_map(|m| m.step())
            .partition(|m| m.done);

        // println!("[DEBUG] machines: ");
        // ms.iter().for_each(|m| {
        //      println!("[DEBUG]   comp: {}", m.comp);
        //      println!("[DEBUG]   stack size: {:?}", m.stack.len());
        //      println!("[DEBUG]   env size: {:?}", m.env.size());
        //      println!("[DEBUG]   lenv size: {:?}", m.lenv.size());
        //      println!("[DEBUG]   senv size: {:?}", m.senv.size())
        //  });

        let mut dones = done.iter();
        while let Some(m) = dones.next() {
            match &*m.comp {
                MComputation::Return(v) => {
                    let out = output(v.clone(), m.env.clone(), &m.lenv, &m.senv);
                    match out {
                        Some(xs) => { println!("> {}", xs); solns += 1 }
                        None => ()
                    }
                },
                _ => unreachable!()
            }
        }
        machines = ms;
    }
    
    println!(">>> {} solutions", solns);
}

fn output(val : Rc<MValue>, env : Rc<Env>, lenv : &LogicEnv, senv : &SuspEnv) -> Option<String> {
    Some(VClosure::Clos { val, env }.close_val(lenv, senv)?.to_string())
}