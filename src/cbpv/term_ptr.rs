use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{cbpv::Term, eval::{Env, LocationsClone}};

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

    pub fn contains_typed_var(&self) -> bool {
        self.val.contains_typed_var()
    }
}

impl LocationsClone for TermPtr {
    fn clone_with_locations(
        &self,
        new_val_locs: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>,
        new_env_locs: &mut HashMap<*mut Env, Rc<RefCell<Env>>>
    ) -> Self {
        TermPtr {
            val: if self.val.contains_typed_var() {
                Rc::new(self.val.clone_with_locations(new_val_locs, new_env_locs))
            } else {
                Rc::clone(&self.val)
            }
        }
    }
}