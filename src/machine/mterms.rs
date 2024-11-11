use std::{fmt::Display, rc::Rc};

use crate::{cbpv::terms::ValueType, machine::vclosure::VClosure};

#[derive(PartialEq, Clone, Debug)]
pub enum MValue {
    Var(usize),
    Zero,
    Succ(Rc<MValue>),
    Pair(Rc<MValue>, Rc<MValue>),
    Inl(Rc<MValue>),
    Inr(Rc<MValue>),
    Nil,
    Cons(Rc<MValue>, Rc<MValue>),
    Thunk(Rc<MComputation>)
}
    
fn print_nat(n : &MValue) -> Option<String> {
    fn print_nat_aux(n : &MValue, i : usize) -> Option<usize> {
        match n {
            MValue::Zero => Some(i),
            MValue::Succ(v) => print_nat_aux(&v, i+1),
            _ => None
        }
    }
    Some(print_nat_aux(n, 0)?.to_string())
}

impl Display for MValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MValue::Var(i) => write!(f, "idx {}", i),
            MValue::Zero => write!(f, "{}", print_nat(&MValue::Zero).expect("foo")),
            MValue::Succ(v) => {
                match print_nat(self) {
                    Some(n) => write!(f, "{}", n),
                    None => write!(f, "Succ({})", v),
                }
            },
            MValue::Nil => write!(f, "Nil"),
            MValue::Cons(v, w) => write!(f, "Cons({}, {})", v, w),
            MValue::Thunk(t) => write!(f, "Thunk({})", t),
            MValue::Pair(v, w) => write!(f, "({}, {})", v, w),
            MValue::Inl(v) => write!(f, "inl({})", v),
            MValue::Inr(w) => write!(f, "inr({})", w)
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MComputation {
    // Value eliminators
    Ifz {
        num : Rc<MValue>,
        zk : Rc<MComputation>,
        sk : Rc<MComputation>
    },
    Match {
        list : Rc<MValue>,
        nilk : Rc<MComputation>,
        consk : Rc<MComputation>
    },
    Case { 
        sum : Rc<MValue>,
        inlk : Rc<MComputation>,
        inrk : Rc<MComputation>
    },
    // CBPV primitives
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
    // FLP
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
    // Recursion
    Rec {
        body : Rc<MComputation>
    },
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
            MComputation::Match { list, nilk, consk } => 
                write!(f, "match({}, {}, {})", list, nilk, consk),
            _ => todo!()
        }
    }
}