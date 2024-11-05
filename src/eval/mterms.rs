use std::rc::Rc;

use crate::cbpv::terms::ValueType;

#[derive(PartialEq, Eq, Clone)]
pub enum MVar {
    Level(usize),
    Index(usize)
}

pub enum MValue {
    Var(MVar),
    Zero,
    Succ(Rc<MValue>),
    Bool(bool),
    Nil,
    Cons(Rc<MValue>, Rc<MValue>),
    Thunk(Rc<MComputation>)
}

impl MValue {
    pub fn occurs(self : &Self, var : &MVar) -> bool {
        match self {
            MValue::Var(s) => *s == *var,
            MValue::Zero => false,
            MValue::Succ(v) => v.clone().occurs(var),
            MValue::Bool(_) => false,
            MValue::Nil => false,
            MValue::Cons(v, w) => v.clone().occurs(var) || w.clone().occurs(var),
            MValue::Thunk(v) => v.clone().occurs(var),
        }
    }
}

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
    }
}

impl MComputation {
    pub fn occurs(self : &Self, var : &MVar) -> bool {
        match self {
            MComputation::Return(v) => v.clone().occurs(var),
            MComputation::Bind { comp, cont } => 
                comp.clone().occurs(var) || cont.clone().occurs(var),
            MComputation::Force(v) => v.clone().occurs(var),
            MComputation::Lambda { body } => 
                body.clone().occurs(var),
            MComputation::App { op, arg } => 
                op.clone().occurs(var) || arg.clone().occurs(var),
            MComputation::Choice(vec) => 
                vec.iter().all(|c| c.clone().occurs(var)),
            MComputation::Exists { ptype, body } =>
                body.clone().occurs(var),
            MComputation::Equate { lhs, rhs, body } => 
                lhs.clone().occurs(var) || rhs.clone().occurs(var) || body.clone().occurs(var),
            MComputation::Ifz { num, zk, sk } => 
                num.clone().occurs(var) || zk.clone().occurs(var) || sk.clone().occurs(var),
        }
    }
}