use std::{fmt::Display, rc::Rc};

#[derive(PartialEq, Clone, Debug)]
pub enum ValueType {
    Nat,
    Bool,
    Pair(Box<ValueType>, Box<ValueType>),
    List(Box<ValueType>),
    Thunk(Box<ComputationType>)
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Nat => write!(f, "Nat"),
            ValueType::Bool => write!(f, "Bool"),
            ValueType::List(value_type) => write!(f, "[{}]", value_type),
            ValueType::Thunk(computation_type) => write!(f, "THONK"),
            ValueType::Pair(value_type, value_type1) => todo!(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
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
    },
}