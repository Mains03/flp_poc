use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::state_term::StateTerm;

#[derive(Debug)]
pub struct Env {
    envs: Vec<HashMap<String, StateTerm>>
}

impl Env {
    pub fn new() -> Self {
        Env { envs: vec![HashMap::new()] }
    }

    pub fn store(&mut self, var: String, val: StateTerm) {
        if self.envs.get(self.envs.len()-1).unwrap().contains_key(&var) {
            self.envs.push(HashMap::new());
        };

        let i = self.envs.len()-1;
        let env = self.envs.get_mut(i).unwrap();
        env.insert(var, val);
    }

    pub fn lookup(&self, var: &String) -> Option<StateTerm> {
        let ret;

        let mut i = self.envs.len()-1;
        loop {
            let env = self.envs.get(i).unwrap();
            match env.get(var) {
                Some(val) => {
                    ret = Some(val.clone());
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

    pub fn expand_value(&self, term: Term) -> StateTerm {
        match term {
            Term::Var(var) => self.lookup(&var).unwrap(),
            Term::Succ(succ) => match self.expand_value(*succ) {
                StateTerm::Term(term) => StateTerm::Term(Term::Succ(Box::new(term))),
                StateTerm::Closure(_) => unreachable!()
            },
            Term::TypedVar(shape) => if shape.borrow().is_some() {
                StateTerm::Term(match shape.borrow().clone().unwrap() {
                    Term::Zero => Term::Zero,
                    Term::Succ(term) => match self.expand_value(*term) {
                        StateTerm::Term(term) => Term::Succ(Box::new(term)),
                        StateTerm::Closure(_) => unreachable!()
                    },
                    _ => unreachable!()
                })
            } else {
                StateTerm::Term(Term::TypedVar(shape))
            }
            _ => StateTerm::Term(term)
        }
    }

    pub fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
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
