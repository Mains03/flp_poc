pub mod mterms;
mod step;
// mod translate;
use mterms::{MComputation, MValue};
use step::{close_val, step, Machine, VClosure};

fn eval(m : Machine, mut fuel : usize) -> Vec<MValue> {

    let mut machines = vec![m];
    let mut values = vec![];
    
    while fuel > 0 && !machines.is_empty() {
        let (mut done, ms) : (Vec<Machine>, Vec<Machine>) = machines.into_iter().flat_map(|m| step(m)).partition(|m| m.done);
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
#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::machine::{eval, mterms::*, step::*};

    use super::step;
    
    #[test]
    fn test1() {
        // Succ(Zero)
        let val = Rc::new(MValue::Succ(MValue::Zero.into()));
        let ret_val : MComputation = MComputation::Return(val.clone());
        let m = Machine { comp: ret_val.into(), env : empty_env(), stack: vec![].into(), done: false };
        let vals = eval(m, 10);
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0], *val);
    }

    #[test]
    fn test2() {
        // this tests the stack
        let val = Rc::new(MValue::Succ(MValue::Zero.into()));
        let id : MComputation = MComputation::Lambda { body: MComputation::Return(MValue::Var(0).into()).into() };
        let app = MComputation::App { op: id.into(), arg: val.clone() };
        println!("this should be (λx.x)1: {}", app);

        let m = Machine { comp: app.into(), env : empty_env(), stack: vec![].into(), done: false };
        let vals = eval(m, 1000);
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0], *val);
        println!("{}", vals[0])
    }
}