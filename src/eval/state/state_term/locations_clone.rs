use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

pub trait LocationsClone {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self;
}