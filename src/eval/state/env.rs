use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::cbpv::Term;

use super::state_term::{locations_clone::LocationsClone, state_term::StateTerm, value::{Value, ValueStore}};

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

impl ValueStore for Env {
    fn store(&mut self, var: String, val: Value) {
        if self.envs.get(self.envs.len()-1).unwrap().contains_key(&var) {
            self.envs.push(HashMap::new());
        };

        let i = self.envs.len()-1;
        let env = self.envs.get_mut(i).unwrap();
        env.insert(var, val.to_state_term());
    }

    fn lookup(&self, var: &String) -> Option<Value> {
        let ret;

        let mut i = self.envs.len()-1;
        loop {
            let env = self.envs.get(i).unwrap();
            match env.get(var) {
                Some(state_term) => {
                    ret = Some(state_term.as_value());
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

    fn expand_value(&self, term: Term) -> Value {
        match term {
            Term::Var(var) => self.lookup(&var).unwrap(),
            Term::Succ(succ) => match self.expand_value(*succ) {
                Value::Term(term) => Value::Term(Term::Succ(Box::new(term))),
                Value::Closure(_) => unreachable!()
            },
            Term::Cons(x, xs) => match self.expand_value(*x) {
                Value::Term(x) => match self.expand_value(*xs) {
                    Value::Term(xs) => Value::Term(Term::Cons(Box::new(x), Box::new(xs))),
                    Value::Closure(_) => unreachable!()
                },
                Value::Closure(_) => unreachable!()
            },
            Term::TypedVar(shape) => if shape.borrow().is_some() {
                Value::Term(match shape.borrow().clone().unwrap() {
                    Term::Zero => Term::Zero,
                    Term::Succ(term) => match self.expand_value(*term) {
                        Value::Term(term) => Term::Succ(Box::new(term)),
                        Value::Closure(_) => unreachable!()
                    },
                    Term::Nil => Term::Nil,
                    Term::Cons(x, xs) => match self.expand_value(*x) {
                        Value::Term(x) => match self.expand_value(*xs) {
                            Value::Term(xs) => Term::Cons(Box::new(x), Box::new(xs)),
                            Value::Closure(_) => unreachable!()
                        },
                        Value::Closure(_) => unreachable!()
                    }
                    _ => unreachable!()
                })
            } else {
                Value::Term(Term::TypedVar(shape))
            }
            _ => Value::Term(term)
        }
    }
}

impl LocationsClone for Env {
    fn clone_with_locations(&self, new_locations: &mut HashMap<*mut Option<Term>, Rc<RefCell<Option<Term>>>>) -> Self {
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
