use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::term_ptr::TermPtr;

use super::state_term::{locations_clone::LocationsClone, state_term::{StateTerm, StateTermStore}};

#[derive(Debug)]
pub struct Env {
    envs: Vec<HashMap<String, StateTerm>>
}

impl Env {
    pub fn new() -> Self {
        Env { envs: vec![HashMap::new()] }
    }

    pub fn release(&mut self, var: &String) {
        let mut i = self.envs.len()-1;
        loop {
            let env = self.envs.get_mut(i).unwrap();

            if env.contains_key(var) {
                env.remove(var);
                break;
            } else {
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            }
        }

        loop {
            if self.envs.get(self.envs.len()-1).unwrap().is_empty() {
                if self.envs.len() > 1 {
                    self.envs.remove(self.envs.len()-1);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}

impl StateTermStore for Env {
    fn store(&mut self, var: String, val: StateTerm) {
        if self.envs.get(self.envs.len()-1).unwrap().contains_key(&var) {
            self.envs.push(HashMap::new());
        };

        let i = self.envs.len()-1;
        let env = self.envs.get_mut(i).unwrap();
        env.insert(var, val);
    }

    fn lookup(&self, var: &String) -> Option<StateTerm> {
        let ret;

        let mut i = self.envs.len()-1;
        loop {
            let env = self.envs.get(i).unwrap();
            match env.get(var) {
                Some(state_term) => {
                    ret = Some(state_term.clone());
                    break;
                },
                None => ()
            }

            if i == 0 {
                ret = None;
                break;
            } else {
                i -=1 ;
            }
        }

        ret
    }
}

impl LocationsClone for Env {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<TermPtr>, Rc<RefCell<Option<TermPtr>>>>) -> Self {
        let mut envs = vec![];

        let mut i = self.envs.len()-1;

        loop {
            let env = self.envs.get(i).unwrap().iter()
                .fold(HashMap::new(), |mut acc, (key, val)| {
                    acc.insert(key.clone(), val.clone_with_locations(new_locations));
                    acc
                });

            envs.push(env);

            if i == 0 {
                break;
            } else {
                i -= 1;
            }
        }

        envs.reverse();

        Self { envs }
    }
}
