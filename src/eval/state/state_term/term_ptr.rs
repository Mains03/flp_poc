use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::locations_clone::LocationsClone;

#[derive(Clone, Debug)]
pub struct TermPtr {
    val: Rc<Term>
}

impl TermPtr {
    pub fn new(term: Term) -> Self {
        TermPtr { val: Rc::new(term) }
    }

    pub fn term(&self) -> Term {
        self.val.as_ref().clone()
    }
}

impl LocationsClone for TermPtr {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
        TermPtr {
            val: if self.val.contains_typed_var() {
                Rc::new(self.val.clone_with_locations(new_locations))
            } else {
                Rc::clone(&self.val)
            }
        }
    }
}