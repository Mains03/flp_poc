use std::{fmt::Display, rc::Rc};

use crate::cbpv::terms::ValueType;

#[derive(PartialEq, Clone, Debug)]
pub enum MValue {
    Var(usize),
    Zero,
    Succ(Rc<MValue>),
    Bool(bool),
    Nil,
    Cons(Rc<MValue>, Rc<MValue>),
    Thunk(Rc<MComputation>)
}

impl Display for MValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MValue::Var(i) => write!(f, "idx {}", i),
            MValue::Zero => write!(f, "Zero"),
            MValue::Succ(v) => write!(f, "Succ {}", *v),
            MValue::Bool(b) => write!(f, "{}", b),
            MValue::Nil => write!(f, "Nil"),
            MValue::Cons(v, w) => write!(f, "Cons({}, {})", v, w),
            MValue::Thunk(t) => write!(f, "Thunk({})", t),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MComputation {
    Return(Rc<MValue>),
    Bind {
        comp: Rc<MComputation>,
        cont: Rc<MComputation>,
    },
    Force(Rc<MValue>),
    Lambda { body: Rc<MComputation> },
    App {
        op: Rc<MComputation>,
        arg: Rc<MValue>
    },
    Choice(Vec<Rc<MComputation>>),
    Exists {
        ptype : ValueType,
        body: Rc<MComputation>
    },
    Equate {
        lhs: Rc<MValue>,
        rhs: Rc<MValue>,
        body: Rc<MComputation>
    },
    Ifz {
        num : Rc<MValue>,
        zk : Rc<MComputation>,
        sk : Rc<MComputation>
    },
    Rec {
        body : Rc<MComputation>
    },
    Match {
        list : Rc<MValue>,
        nilk : Rc<MComputation>,
        consk : Rc<MComputation>
    }
}

impl Display for MComputation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MComputation::Return(v) => write!(f, "return({})", v),
            MComputation::Bind { comp, cont } => write!(f, "{} to {}", comp, cont),
            MComputation::Force(v) => write!(f, "force({})", v),
            MComputation::Lambda { body } => write!(f, "λ({})", body),
            MComputation::App { op, arg } => write!(f, "{}({})", op, arg),
            MComputation::Choice(vec) => {
                vec.iter().map(|c| write!(f, "{} []", c)).last().expect("lol")
            },
            MComputation::Exists { ptype, body } => 
                write!(f, "exists {}. {}", ptype, body),
            MComputation::Equate { lhs, rhs, body } => 
                write!(f, "{} =:= {}. {}", lhs, rhs, body),
            MComputation::Ifz { num, zk, sk } => 
                write!(f, "ifz({}, {}, {})", num, zk, sk),
            MComputation::Rec { body } => write!(f, "rec({})", body),
            MComputation::Match { list, nilk, consk } => todo!(),
        }
    }
}