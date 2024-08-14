use std::collections::HashMap;

use crate::parser::syntax::r#type::Type;

#[derive(Clone, Debug)]
pub struct Vars {
    vars: HashMap<String, Type>
}

impl Vars {
    pub fn new() -> Self {
        Vars { vars: HashMap::new() }
    }

    pub fn bind(&mut self, var: String, r#type: Type) {
        todo!()
    }

    pub fn get_type(&self, var: &String) -> Option<Type> {
        todo!()
    }
}