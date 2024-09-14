use std::collections::HashSet;

use crate::parser::syntax::arg::Arg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeVars {
    vars: HashSet<String>
}

impl FreeVars {
    pub fn new() -> Self {
        FreeVars { vars: HashSet::new() }
    }

    pub fn from_vars(vars: Vec<String>) -> Self {
        FreeVars { vars: HashSet::from_iter(vars) }
    }

    pub fn extend(&mut self, vars: FreeVars) {
        self.vars.extend(vars.vars);
    }

    pub fn add_var(&mut self, var: String) {
        self.vars.insert(var);
    }

    pub fn remove_var(&mut self, var: &String) {
        self.vars.remove(var);
    }

    pub fn remove_arg(&mut self, arg: &Arg) {
        match arg {
            Arg::Ident(var) => { self.vars.remove(var); },
            Arg::Pair(lhs, rhs) => {
                self.remove_arg(&lhs);
                self.remove_arg(&rhs);
            }
        }
    }

    pub fn vars(&self) -> &HashSet<String> {
        &self.vars
    }
}