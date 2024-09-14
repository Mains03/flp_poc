use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{cbpv::Term, eval::LocationsClone};

use super::free_vars::FreeVars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermPtr {
    val: Rc<Term>
}

impl TermPtr {
    pub fn from_term(term: Term) -> Self {
        TermPtr { val: Rc::new(term) }
    }

    pub fn term(&self) -> &Term {
        self.val.as_ref()
    }

    pub fn free_vars(&self) -> FreeVars {
        self.val.free_vars()
    }

    pub fn contains_typed_var(&self) -> bool {
        self.val.contains_typed_var()
    }
}

impl LocationsClone for TermPtr {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>) -> Self {
        TermPtr {
            val: if self.val.contains_typed_var() {
                Rc::new(self.val.clone_with_locations(new_locations))
            } else {
                Rc::clone(&self.val)
            }
        }
    }
}