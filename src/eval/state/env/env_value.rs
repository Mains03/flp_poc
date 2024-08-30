use std::{cell::RefCell, rc::Rc};

use crate::{cbpv::Term, eval::state::state_term::StateTerm};

#[derive(Clone, Debug)]
pub enum EnvValue {
    Term(StateTerm),
    Type(Rc<RefCell<TypeVal>>),
}

#[derive(Clone, Debug)]
pub struct TypeVal {
    pub val: Option<Shape>
}

#[derive(Clone, Debug)]
pub enum Shape {
    Zero,
    Succ(Rc<RefCell<TypeVal>>)
}

impl TypeVal {
    pub fn to_term(&self) -> Option<Term> {
        match &self.val {
            Some(shape) => match shape {
                Shape::Zero => Some(Term::Zero),
                Shape::Succ(succ) => match succ.borrow().to_term() {
                    Some(term) => Some(Term::Succ(Box::new(term))),
                    None => None
                }
            },
            None => None
        }
    }

    pub fn set_shape(&mut self, shape: &Term)  {
        self.val = Some(TypeVal::set_shape_helper(shape));
    }

    fn set_shape_helper(shape: &Term) -> Shape {
        match shape {
            Term::Succ(succ) => Shape::Succ(Rc::new(RefCell::new(TypeVal {
                val: Some(TypeVal::set_shape_helper(succ))
            }))),
            Term::Zero => Shape::Zero,
            _ => unreachable!()
        }
    }
}
