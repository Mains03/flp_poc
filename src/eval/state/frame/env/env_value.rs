use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{cbpv::Term, eval::state::state_term::StateTerm};

#[derive(Clone, Debug)]
pub enum EnvValue {
    Term(StateTerm),
    Type(Rc<RefCell<TypeVal>>),
}

#[derive(Debug)]
pub struct TypeVal {
    pub val: Option<Shape>
}

#[derive(Clone, Debug)]
pub enum Shape {
    Zero,
    Succ(Rc<RefCell<TypeVal>>)
}

impl EnvValue {
    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>>) -> Self {
        match self {
            EnvValue::Term(term) => EnvValue::Term(term.clone_with_locations(new_locations)),
            EnvValue::Type(r#type) => match new_locations.get(&r#type.as_ptr()) {
                Some(new_location) => EnvValue::Type(Rc::clone(new_location)),
                None => {
                    let val = Rc::new(RefCell::new(r#type.borrow().clone_with_locations(new_locations)));
                    new_locations.insert(r#type.as_ptr(), Rc::clone(&val));
                    EnvValue::Type(val)
                }
            }
        }
    }
}

impl TypeVal {
    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>>) -> Self {
        match &self.val {
            Some(shape) => TypeVal { val: Some(shape.clone_with_locations(new_locations)) },
            None => TypeVal { val: None }
        }
    }

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

impl Shape {
    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>>) -> Self {
        match self {
            Shape::Zero => Shape::Zero,
            Shape::Succ(location) => match new_locations.get(&location.as_ptr()) {
                Some(new_location) => Shape::Succ(Rc::clone(new_location)),
                None => {
                    let new_location = Rc::new(RefCell::new(
                        location.borrow().clone_with_locations(new_locations)
                    ));
                    new_locations.insert(location.as_ptr(), Rc::clone(&new_location));
                    Shape::Succ(new_location)
                }
            }
        }
    }
}