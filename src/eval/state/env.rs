use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::term_ptr::TermPtr;

use super::state_term::{locations_clone::LocationsClone, state_term::{StateTerm, StateTermStore}};

#[derive(Clone, Debug)]
pub struct Env {
    env: HashMap<String, StateTerm>,
    prev: Option<Rc<RefCell<Env>>>
}

impl Env {
    pub fn new() -> Self {
        Env { env: HashMap::new(), prev: None }
    }

    pub fn release(&mut self, var: &String) {
        self.env.remove(var);

        if self.env.is_empty() && self.prev.is_some() {
            let env = match &self.prev {
                Some(env) => env.borrow().clone(),
                None => unreachable!()
            };

            self.env = env.env;
            self.prev = env.prev;
        }
    }
}

impl StateTermStore for Env {
    fn store(&mut self, var: String, val: StateTerm) {
        if self.env.contains_key(&var) {
            self.prev = Some(Rc::new(RefCell::new(self.clone())));
            self.env = HashMap::new();
        }
        
        self.env.insert(var, val);
    }

    fn lookup(&self, var: &String) -> StateTerm {
        if self.env.contains_key(var) {
            self.env.get(var).unwrap().clone()
        } else {
            let mut env = Rc::clone(match &self.prev {
                Some(prev) => prev,
                None => unreachable!()
            });

            let ret_val;
            loop {
                let new_env;
                match env.borrow().env.get(var) {
                    Some(val) => {
                        ret_val = val.clone();
                        break;
                    },
                    None => {
                        match &env.borrow().prev {
                            Some(env) => new_env = Rc::clone(env),
                            None => unreachable!()
                        }
                    }
                }
                env = new_env;
            }

            ret_val
        }
    }
}

impl LocationsClone for Env {
    fn clone_with_locations(
        &self,
        new_val_locs: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>,
        new_env_locs: &mut HashMap<*mut Env, Rc<RefCell<Env>>>
    ) -> Self {
        let env = self.env.iter()
            .fold(HashMap::new(), |mut env, (var, val)| {
                env.insert(var.clone(), val.clone_with_locations(new_val_locs, new_env_locs));
                env
            });

        let prev = match &self.prev {
            Some(prev) => match new_env_locs.get(&prev.as_ptr()) {
                Some(prev) => Some(Rc::clone(prev)),
                None => {
                    let new_prev = Rc::new(RefCell::new(
                        prev.borrow().clone_with_locations(new_val_locs, new_env_locs)
                    ));

                    new_env_locs.insert(prev.as_ptr(), Rc::clone(&new_prev));

                    Some(new_prev)
                }
            },
            None => None
        };

        Self { env, prev }
    }
}
