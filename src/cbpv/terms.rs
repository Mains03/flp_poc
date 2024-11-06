use std::rc::Rc;

#[derive(PartialEq, Eq, Clone)]
pub enum ValueType {
    Nat,
    Bool,
    List(Box<ValueType>),
    Thunk(Box<ComputationType>)
}

#[derive(PartialEq, Eq, Clone)]
pub enum ComputationType {
    Return(Box<ValueType>),
    Arrow(Box<ValueType>, Box<ComputationType>)
}

pub enum Value {
    Var(String),
    Zero,
    Succ(Rc<Value>),
    Bool(bool),
    Nil,
    Cons(Rc<Value>, Rc<Value>),
    Thunk(Rc<Computation>)
}

pub enum Computation {
    Return(Rc<Value>),
    Bind {
        var : String,
        comp: Rc<Computation>,
        cont: Rc<Computation>,
    },
    Force(Rc<Value>),
    Lambda {
        var: String,
        body: Rc<Computation>
    },
    App {
        op: Rc<Computation>,
        arg: Rc<Value>
    },
    Choice(Vec<Rc<Computation>>),
    Exists {
        var : String,
        ptype : ValueType,
        body: Rc<Computation>
    },
    Equate {
        lhs: Rc<Value>,
        rhs: Rc<Value>,
        body: Rc<Computation>
    },
    Ifz {
        num : Rc<Value>,
        zk : Rc<Computation>,
        sk : Rc<Computation>
    }
}

// #[derive(PartialEq, Eq)]
// enum BindingType { Lambda, Exists }

// fn convert_val_aux(val : Rc<Value<String>>, lcount : usize, ecount : usize, env : &mut Vec<(BindingType, String)>) -> Value<MachineVar> {
//     match *val {
//         Value::Var(xs) => 
//             if let Some(i) = env.iter().rposition(|y| xs == y.1) {
//                match &env[i] {
//                 (Lambda, _) => {
//                     if let Some(j) = env.iter().filter(|&y| BindingType::Exists == y.0).rposition(|y| xs == y.1) {
//                         return Value::Var(MachineVar::Level(j))
//                     } else { unreachable!() }
//                 },
//                 (Exists, _) => {
//                     if let Some(j) = env.iter().filter(|&y| BindingType::Lambda == y.0).rposition(|y| xs == y.1) {
//                         return Value::Var(MachineVar::Index(lcount - j - 1))
//                     } else { unreachable!() }
//                 }
//                }
//                todo!()
//             }
//             else { panic!("unbound variable") }
//         Value::Zero => Value::Zero,
//         Value::Succ(v) => Value::Succ(Rc::new(convert_val_aux(v, lcount, ecount, env))),
//         Value::Bool(b) => Value::Bool(b),
//         Value::Nil => Value::Nil,
//         Value::Cons(v, w) => 
//             Value::Cons(Rc::new(convert_val_aux(v, lcount, ecount, env)), Rc::new(convert_val_aux(v, lcount, ecount, env))),
//         Value::Thunk(c) => Value::Thunk(convert_comp_aux(c, lcount, ecount, env).into()),
//     }
// }

// pub fn convert_comp(comp : Rc<Computation<String>>) -> Computation<MachineVar> {
//     convert_comp_aux(comp, 0, 0, &mut vec![])
// }

// fn convert_comp_aux(comp : Rc<Computation<String>>, lcount : usize, ecount: usize, env) -> Computation<MachineVar> {
//     match *comp {
//         Computation::Return(val) => Computation::Return(convert_val_aux(val, lcount, ecount, env).into()),
//         Computation::Bind { var, comp, cont } => 
//             Computation::Bind { },
//         Computation::Force(rc) => todo!(),
//         Computation::Lambda { var, body } => todo!(),
//         Computation::App { op, arg } => todo!(),
//         Computation::Choice(vec) => todo!(),
//         Computation::Exists { var, ptype, body } => todo!(),
//         Computation::Equate { lhs, rhs, body } => todo!(),
//         Computation::Ifz { num, zk, sk } => todo!(),
//     }
// }
