use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::term_ptr::TermPtr;

pub trait LocationsClone {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>) -> Self;
}