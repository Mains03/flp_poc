pub mod mterms;
mod vclosure;
mod lvar;
mod env;
mod unify;
mod step;
mod union_find;
pub mod translate;
use std::{io::Write, rc::Rc};
use env::Env;
use im::vector::Vector;
use lvar::LogicEnv;
use mterms::{MComputation, MValue};
use step::{empty_stack, Machine};
use vclosure::VClosure;
use std::io::stdout;

pub type Ident = usize;

pub fn eval(comp : MComputation, env : Rc<Env>, mut fuel : usize) {

    // println!("[DEBUG] main stmt: {}", comp.clone()) ;
    let m = Machine { comp: comp.into() , env: env.clone(), stack: empty_stack().into(), lenv : LogicEnv::new().into(), done: false };
    let mut machines = vec![m];
    let mut solns = 0;
    while fuel > 0 && !machines.is_empty() {
        let (done, ms) : (Vec<Machine>, Vec<Machine>) = machines.into_iter()
            .flat_map(|m| m.step())
            .partition(|m| m.done);
        // println!("[DEBUG] machines: ");
        // ms.iter().for_each(|m| {
        //      println!("[DEBUG]   comp: {}", m.comp);
        //      println!("[DEBUG]   stack size: {:?}", m.stack.len());
        //      println!("[DEBUG]   env size: {:?}", m.env.size());
        //      println!("[DEBUG]   lenv size: {:?}", m.lenv.size())
        //  });
        let mut dones = done.iter();
        while let Some(m) = dones.next() {
            match &*m.comp {
                MComputation::Return(v) => {
                    let out = output(v.clone(), m.env.clone(), &m.lenv);
                    match out {
                        Some(xs) => { println!("> {}", xs); solns += 1 }
                        None => ()
                    }
                },
                _ => unreachable!()
            }
        }
        machines = ms;
        fuel -= 1
    }
    
    println!(">>> {} solutions", solns);
    
    // values.iter().flat_map(|m| {
    //     match &*m.comp {
    //         MComputation::Return(v) => VClosure::Clos { val: v.clone(), env: m.env.clone() }.close_val(&m.lenv),
    //         _ => unreachable!()
    //     }
    // }).collect()
}

fn output(val : Rc<MValue>, env : Rc<Env>, lenv : &LogicEnv) -> Option<String> {
    Some(VClosure::Clos { val, env }.close_val(lenv)?.to_string())
}