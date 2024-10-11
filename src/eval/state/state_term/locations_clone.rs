use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{cbpv::term_ptr::TermPtr, eval::state::env::Env};

pub trait LocationsClone {
    fn clone_with_locations(
        &self,
        new_val_locs: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>,
        new_env_locs: &mut HashMap<*mut Env, Rc<RefCell<Env>>>
    ) -> Self;
}