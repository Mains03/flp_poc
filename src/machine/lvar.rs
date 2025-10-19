use std::{cell::RefCell, ptr, rc::Rc};

use im::HashMap;

use crate::machine::value_type::ValueType;

use super::{env::Env, mterms::MValue, union_find::UnionFind, Ident, VClosure};

#[derive(Clone)]
pub struct LogicEnv {
    map : HashMap<Ident, (ValueType, Option<Rc<VClosure>>)>,
    union_vars : UnionFind,
    next : usize
}

impl LogicEnv {

    pub fn new() -> LogicEnv {
        LogicEnv {
            map : HashMap::new(),
            union_vars : UnionFind::new(),
            next : 0 
        }
    }
    
    pub fn size(&self) -> usize { self.map.len() }

    pub fn fresh(&mut self, ptype : ValueType) -> Ident {
        let next = self.next;
        self.union_vars.register(self.next);
        self.map.insert(next, (ptype, None));
        // println!("generated {} of type {}", next, ptype);
        self.next = next + 1;
        next
    }
    
    pub fn lookup(&self, ident : Ident) -> Option<Rc<VClosure>> {
        let root = self.union_vars.find(ident);
        if let Some((_, Some(vclos))) = self.map.get(&root) { 
            // println!("[DEBUG] looked up {} to be {}", ident, vclos.clone().val());
            return Some(vclos.clone())
        }
        else { 
            // println!("[DEBUG] LENV failed to find {}", ident);
            return None
         }
    }
    
    pub fn set_vclos(&mut self, ident : Ident, vclos : VClosure) {
        let ptype = self.get_type(ident);
        // println!("[DEBUG] setting {} to be {}", ident, vclos.val());
        self.map.insert(ident, (ptype, Some(vclos.into())));
        self.lookup(ident);
    }
    
    pub fn get_type(&self, ident : Ident) -> ValueType {
        if let Some((ptype, _)) = self.map.get(&ident) { 
            return ptype.clone()
        } 
        else { unreachable!() }
    }
    
    pub fn identify(&mut self, ident1 : Ident, ident2 : Ident) {
        self.union_vars.union(ident1, ident2);
    }
}