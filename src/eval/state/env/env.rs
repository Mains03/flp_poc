use std::collections::HashMap;

use crate::{eval::state::state_term::StateTerm, parser::syntax::r#type::Type};

use super::env_value::EnvValue;

#[derive(Debug)]
pub struct Env {
    envs: Vec<HashMap<String, EnvValue>>
}

impl Env {
    pub fn new() -> Self {
        Env { envs: vec![HashMap::new()] }
    }

    pub fn store(&mut self, var: String, val: StateTerm) {
        let val = EnvValue::Term(val);

        if self.envs.get(self.envs.len()-1).unwrap().contains_key(&var) {
            self.envs.push(HashMap::new());
        };

        let i = self.envs.len()-1;
        let env = self.envs.get_mut(i).unwrap();
        env.insert(var, val);
    }

    pub fn bind(&mut self, _: String, _: Type) {
        todo!()
    }

    pub fn lookup(&self, var: &String) -> Option<EnvValue> {
        let mut i = self.envs.len()-1;
        let ret;
        loop {
            let env = self.envs.get(i).unwrap();
            match env.get(var) {
                Some(val) => {
                    ret = Some(val.clone());
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
                self.envs.remove(self.envs.len()-1);
            } else {
                break;
            }
        }
    }
}