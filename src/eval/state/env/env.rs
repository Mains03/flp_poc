use std::{cell::RefCell, collections::{HashMap, VecDeque}, rc::Rc};

use crate::eval::state::{env_lookup::EnvLookup, state_term::StateTerm};

use super::env_value::{EnvValue, Shape, TypeVal};

#[derive(Debug)]
pub struct Env {
    envs: Vec<HashMap<String, EnvValue>>
}

impl Env {
    pub fn new() -> Self {
        Env { envs: vec![HashMap::new()] }
    }

    pub fn store(&mut self, var: String, val: StateTerm) {
        self.store_env_val(var, EnvValue::Term(val));
    }

    pub fn bind(&mut self, var: String, val: &Rc<RefCell<TypeVal>>) {
        self.store_env_val(
            var, EnvValue::Type(Rc::clone(val))
        );
    }

    fn store_env_val(&mut self, var: String, val: EnvValue) {
        if self.envs.get(self.envs.len()-1).unwrap().contains_key(&var) {
            self.envs.push(HashMap::new());
        };

        let i = self.envs.len()-1;
        let env = self.envs.get_mut(i).unwrap();
        env.insert(var, val);
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

impl EnvLookup for Env {
    fn lookup(&self, var: &String) -> Option<EnvValue> {
        let mut i = self.envs.len()-1;
        let ret;
        loop {
            let env = self.envs.get(i).unwrap();
            match env.get(var) {
                Some(val) => {
                    ret = Some(match val {
                        EnvValue::Term(_) => val.clone(),
                        EnvValue::Type(r#type) => EnvValue::Type(Rc::clone(r#type))
                    });
                    break;
                },
                None => if i == 0 {
                    ret = None;
                    break;
                } else {
                    i -= 1;
                }
            }
        }

        ret
    }
}

impl Clone for Env {
    fn clone(&self) -> Self {
        let mut envs = vec![];

        let mut i = self.envs.len()-1;
        let mut new_locations: HashMap<*mut TypeVal, Rc<RefCell<TypeVal>>> = HashMap::new();

        loop {
            let env = self.envs.get(i).unwrap();
            let mut env_clone = HashMap::new();

            let mut queue = VecDeque::new();
            env.iter()
                .for_each(|(key, val)| {
                    queue.push_back((key.clone(), val.clone()));
                });

            loop {
                match queue.pop_front() {
                    Some((key, val)) => match val.clone() {
                        EnvValue::Term(term) => { env_clone.insert(key.clone(), EnvValue::Term(term.clone())); },
                        EnvValue::Type(r#type) => match new_locations.get(&r#type.as_ptr()) {
                            Some(new_location) => {
                                let env_value = EnvValue::Type(Rc::clone(new_location));
                                env_clone.insert(key.clone(), env_value);
                            },
                            None => match r#type.borrow().val.clone() {
                                Some(shape) => match shape {
                                    Shape::Zero => {
                                        let val = Rc::new(RefCell::new(TypeVal { val: Some(Shape::Zero) }));
                                        new_locations.insert(r#type.as_ptr(), Rc::clone(&val));
                                        env_clone.insert(key.clone(), EnvValue::Type(Rc::clone(&val)));
                                    },
                                    Shape::Succ(succ) => match new_locations.get(&succ.as_ptr()) {
                                        Some(new_location) => {
                                            let val = Rc::new(RefCell::new(TypeVal {
                                                val: Some(Shape::Succ(Rc::clone(new_location)))
                                            }));
                                            new_locations.insert(r#type.as_ptr(), Rc::clone(&val));
                                            env_clone.insert(key.clone(), EnvValue::Type(Rc::clone(&val)));
                                        },
                                        None => { queue.push_back((key, val)); }
                                    }
                                },
                                None => {
                                    let val = Rc::new(RefCell::new(TypeVal { val: None }));
                                    new_locations.insert(r#type.as_ptr(), Rc::clone(&val));
                                    env_clone.insert(key.clone(), EnvValue::Type(Rc::clone(&val)));
                                }
                            },
                        }
                    },
                    None => break
                }
            }

            envs.push(env_clone);

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