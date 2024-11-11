pub mod mterms;
mod vclosure;
mod lvar;
mod env;
mod unify;
mod step;
mod union_find;
pub mod translate;
use std::{io::Write, os::windows::io, rc::Rc};
use env::Env;
use im::vector::Vector;
use lvar::LogicEnv;
use mterms::{MComputation, MValue};
use step::{empty_stack, step, Machine};
use vclosure::VClosure;
use std::io::stdout;

pub type Ident = usize;

pub fn eval(comp : MComputation, env : Rc<Env>, mut fuel : usize) {

    let m = Machine { comp: comp.into() , env: env.clone(), stack: empty_stack().into(), lenv : LogicEnv::new().into(), done: false };
    let mut machines = vec![m];
    
    print!("[");
    while fuel > 0 && !machines.is_empty() {
        let (mut done, ms) : (Vec<Machine>, Vec<Machine>) = machines.into_iter().flat_map(|m| step(m)).partition(|m| m.done);
        // println!("[DEBUG] machines: ");
        // ms.iter().for_each(|m| {
        //     println!("[DEBUG]   comp: {}", m.comp);
        //     println!("[DEBUG]   stack size: {:?}", m.stack.len());
        //     println!("[DEBUG]   env size: {:?}", m.env.size())
        // });
        let mut dones = done.iter().peekable();
        while let Some(m) = dones.next() {
            match &*m.comp {
                MComputation::Return(v) => {
                    let out = output(v.clone(), m.env.clone(), &m.lenv);
                    match out {
                        Some(xs) => print!("{}", xs),
                        None => ()
                    }
                    if dones.peek().is_some() { println!(", ") }
                    stdout().flush().expect("Failed to flush stdout");
                },
                _ => unreachable!()
            }
        }
        machines = ms;
        fuel -= 1
    }
    print!("]");
    
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