use std::rc::Rc;
use im::Vector;
use super::Ident;
use super::{mterms::MValue, VClosure};

#[derive(Clone, Debug)]
pub struct Env {
    vec : Vector<VClosure>
}

impl Env {

    pub fn empty() -> Rc<Env> { 
        Env { vec : Vector::new() }.into()
    }

    pub fn lookup(&self, i : usize) -> Option<&VClosure> {
        self.vec.get(i).map(|v| &*v)
    }
    
    fn extend(&self, vclos : VClosure) -> Env {
        let mut vec = self.vec.clone();
        vec.push_front(vclos);
        Env { vec }
    }

    pub fn extend_val(&self, val : Rc<MValue>, env : Rc<Env>) -> Rc<Env> {
        self.extend( VClosure::Clos { val, env }).into()
    }

    pub fn extend_lvar(&self, ident : Ident) -> Rc<Env> {
        self.extend(VClosure::LogicVar { ident }).into()
    }

    pub fn extend_susp(&self, ident : Ident) -> Rc<Env> {
        self.extend(VClosure::Susp { ident }).into()
    }
}