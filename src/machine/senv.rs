use std::rc::Rc;

use im::HashMap;

use crate::cbpv::terms::ValueType;

use super::{env::Env, mterms::{MComputation, MValue}, union_find::UnionFind, ComputationInEnv, Ident, VClosure, ValueInEnv};


#[derive(Clone)]
pub struct SuspEnv {
    map : HashMap<Ident, Result<ValueInEnv, ComputationInEnv>>,
    next : usize
}

impl SuspEnv {

    pub fn new() -> SuspEnv {
        SuspEnv {
            map : HashMap::new(),
            next : 0 
        }
    }
    
    pub fn size(&self) -> usize { self.map.len() }

    pub fn fresh(&mut self, c : Rc<MComputation>, env : Rc<Env>) -> Ident {
        let next = self.next;
        self.map.insert(next, Err((c, env)));
        self.next = next + 1;
        next
    }
    
    pub fn lookup(&self, ident : &Ident) -> &Result<ValueInEnv, ComputationInEnv>{
        self.map.get(&ident).expect("unknown suspension ident")
    }
    
    pub fn set(&mut self, ident : &Ident, val : Rc<MValue>, env : Rc<Env>) {
        self.map.insert(*ident, Ok((val, env)));
        self.lookup(ident);
    }
}