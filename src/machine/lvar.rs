use std::{cell::RefCell, ptr, rc::Rc};

use im::HashMap;

use union_find::{QuickFindUf, UnionBySize, UnionFind};

use crate::cbpv::terms::ValueType;

use super::{env::Env, mterms::MValue, Ident, VClosure};

#[derive(Clone)]
pub struct LogicEnv {
    map : HashMap<Ident, (ValueType, Option<Rc<VClosure>>)>,
    union_vars : QuickFindUf::<UnionBySize>,
    next : usize
}

impl LogicEnv {

    pub fn new() -> LogicEnv {
        LogicEnv {
            map : HashMap::new(),
            union_vars : QuickFindUf::new(100),
            next : 0 
        }
    }

    // pub fn split(self : Rc<Self>) -> Rc<Self> {
    //     LogicEnv {
    //         map : self.map.clone(),
    //         next : self.next
    //     }.into()
    // }
    
    pub fn fresh(&mut self, ptype : ValueType) -> Ident {
        let next = self.next;
        self.map.insert(next, (ptype, None));
        self.next = next + 1;
        next
    }
    
    pub fn lookup(&self, ident : &Ident) -> Option<Rc<VClosure>> {
        // let root = self.union_vars.find(*ident);
        if let Some((_, Some(vclos))) = self.map.get(ident) { 
            return Some(vclos.clone())
        }
        else { return None }
    }
    
    pub fn set_vclos(&mut self, ident : &Ident, vclos : &Rc<VClosure>) {
        let ptype = self.get_type(ident);
        self.map.insert(*ident, (ptype, Some(vclos.clone())));
    }
    
    pub fn get_type(&self, ident : &Ident) -> ValueType {
        if let Some((ptype, _)) = self.map.get(ident) { return ptype.clone() }
        else { panic!() }
    }
    
    pub fn identify(&mut self, ident1 : &Ident, ident2 : &Ident) {
        self.union_vars.union(*ident1, *ident2);
    }
}