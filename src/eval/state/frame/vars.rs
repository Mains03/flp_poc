use std::collections::HashMap;

use crate::parser::syntax::r#type::Type;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vars {
    vars: HashMap<String, Type>
}

impl Vars {
    pub fn new() -> Self {
        Vars { vars: HashMap::new() }
    }

    pub fn bind(&mut self, var: String, r#type: Type) {
        self.vars.insert(var, r#type);
    }

    pub fn get_type(&self, var: &String) -> Option<Type> {
        match self.vars.get(var) {
            Some(r#type) => Some(r#type.clone()),
            None => None
        }
    }
}