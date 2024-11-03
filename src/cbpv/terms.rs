use std::rc::Rc;

use super::pm::PM;
use crate::parser::syntax::r#type::Type;

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
        ptype : Type,
        body: Rc<Computation>
    },
    Equate {
        lhs: Rc<Value>,
        rhs: Rc<Value>,
        body: Rc<Computation>
    },
    PM(PM)
}