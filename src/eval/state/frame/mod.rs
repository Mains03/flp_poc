use env::Env;
use vars::Vars;

use crate::{cbpv::Term, parser::syntax::r#type::Type};

mod env;

mod vars;

#[derive(Clone, Debug)]
pub struct Frame {
    vars: Vars,
    env: Env,
    prev: Option<Box<Frame>>
}

pub enum LookupResult {
    Type(Type),
    Term(Term)
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            vars: Vars::new(),
            env: Env::new(),
            prev: None
        }
    }

    pub fn push(old: Frame) -> Self {
        Frame {
            vars: Vars::new(),
            env: Env::new(),
            prev: Some(Box::new(old))
        }
    }

    pub fn pop(self) -> Self {
        *self.prev.unwrap()
    }

    pub fn bind(&mut self, var: String, r#type: Type) {
        self.vars.bind(var, r#type);
    }

    pub fn store(&mut self, var: String, val: Term) {
        self.env.store(&var, val);
    }

    pub fn lookup(&self, var: &String) -> LookupResult {
        let mut var = var.clone();
        let mut frame = self;
        let result;
        loop {
            match frame.env.get_value(&var) {
                Some(term) => match term {
                    Term::Var(v) => var = v.clone(),
                    term => {
                        result = LookupResult::Term(term);
                        break;
                    }
                },
                None => match frame.vars.get_type(&var) {
                    Some(r#type) => {
                        result = LookupResult::Type(r#type);
                        break;
                    },
                    None => match &frame.prev {
                        Some(new_frame) => frame = &new_frame,
                        None => unreachable!()
                    }
                }
            }
        }

        result
    }
}