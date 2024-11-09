use std::rc::Rc;

use im::Vector;

use super::lvar::LogicVar;
use super::{mterms::MValue, VClosure};

#[derive(Clone, Debug)]
pub struct Env {
    vec : Vector<Rc<VClosure>>
}

impl Env {
    pub fn empty() -> Rc<Env> { 
        Env { vec : Vector::new() }.into()
    }

    pub fn lookup(&self, i : usize) -> &VClosure {
        self.vec.get(i).expect("indexing error")
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

    pub fn extend_lvar(&self, lvar : Rc<LogicVar>) -> Rc<Env> {
        self.extend(VClosure::LogicVar { lvar }.into()).into()
    }
    
    pub fn has_unresolved_lvars(&self) -> bool {
        let vec = &self.vec;
        vec.iter().any(|vclos| (**vclos).has_unresolved_lvars())
    }

    pub fn deep_clone(self : &Rc<Self>) -> Rc<Env> {
        if self.has_unresolved_lvars() {
            let mut new_env = self.vec.clone();
            for vclos in new_env.iter_mut() {
                if vclos.has_unresolved_lvars() {
                    *vclos = vclos.deep_clone()
                }
            }
            Rc::new(Env { vec : new_env })
        }
        else {
            self.clone()
        }
    }
}