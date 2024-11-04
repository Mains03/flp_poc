use std::rc::Rc;

use super::pm::PM;
use crate::parser::syntax::r#type::Type;


#[derive(PartialEq, Clone)]
pub enum ValueType {
    Nat,
    Bool,
    List(Box<ValueType>),
    Thunk(Box<ComputationType>)
}

#[derive(PartialEq, Clone)]
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

impl Value {
    pub fn occurs(self : &Self, var : &String) -> bool {
        match self {
            Value::Var(s) => *s == *var,
            Value::Zero => false,
            Value::Succ(v) => v.clone().occurs(var),
            Value::Bool(_) => false,
            Value::Nil => false,
            Value::Cons(v, w) => v.clone().occurs(var) || w.clone().occurs(var),
            Value::Thunk(v) => v.clone().occurs(var),
        }
    }
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

impl Computation {
    pub fn occurs(self : &Self, var : &String) -> bool {
        match self {
            Computation::Return(v) => v.clone().occurs(var),
            Computation::Bind { var: s, comp, cont } => 
                *s != *var && (comp.clone().occurs(var) || cont.clone().occurs(var)),
            Computation::Force(v) => v.clone().occurs(var),
            Computation::Lambda { var: s, body } => 
                *s != *var && body.clone().occurs(var),
            Computation::App { op, arg } => 
                op.clone().occurs(var) || arg.clone().occurs(var),
            Computation::Choice(vec) => 
                vec.iter().all(|c| c.clone().occurs(var)),
            Computation::Exists { var: s, ptype, body } => todo!(),
            Computation::Equate { lhs, rhs, body } => 
              lhs.clone().occurs(var) || rhs.clone().occurs(var) || body.clone().occurs(var),
            Computation::Ifz { num, zk, sk } => 
              num.clone().occurs(var) || zk.clone().occurs(var) || sk.clone().occurs(var),
        }
    }
}