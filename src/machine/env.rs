use std::rc::Rc;

use im::Vector;

use super::Ident;
use super::{mterms::MValue, VClosure};

#[derive(Clone, PartialEq, Debug)]
pub struct Env {
    vec : Vector<Rc<VClosure>>
}

impl Env {
    pub fn empty() -> Rc<Env> { 
        Env { vec : Vector::new() }.into()
    }

    pub fn lookup(&self, i : usize) -> Option<&VClosure> {
        self.vec.get(i).map(|v| &**v)
    }
    
    pub fn size(&self) -> usize {
        self.vec.len()
    }
    
    fn extend(&self, vclos : Rc<VClosure>) -> Env {
        let mut vector = self.vec.clone();
        vector.push_front(vclos);
        Env { vec : vector }
    }

    pub fn extend_clos(&self, val : Rc<MValue>, venv : Rc<Env>) -> Rc<Env> {
        self.extend( VClosure::Clos { val, env : venv }.into()).into()
    }

    pub fn extend_lvar(&self, ident : Ident) -> Rc<Env> {
        self.extend(VClosure::LogicVar { ident }.into()).into()
    }
}