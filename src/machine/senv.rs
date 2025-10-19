use std::rc::Rc;

use im::HashMap;

use super::{env::Env, mterms::{MComputation, MValue}, union_find::UnionFind, Ident, VClosure};

type CClosure = (Rc<MComputation>, Rc<Env>);

#[derive(Clone)]
pub struct SuspEnv {
    map : HashMap<Ident, Result<VClosure, CClosure>>,
    next : usize
}

#[derive(Clone, Debug)]
pub struct SuspAt {
    pub ident : Ident,
    pub comp : Rc<MComputation>,
    pub env : Rc<Env>
}

impl SuspEnv {

    pub fn new() -> SuspEnv {
        SuspEnv {
            map : HashMap::new(),
            next : 0 
        }
    }
    
    pub fn size(&self) -> usize { self.map.len() }

    pub fn fresh(&mut self, comp : &Rc<MComputation>, env : &Rc<Env>) -> Ident {
        let next = self.next;
        self.map.insert(next, Err((comp.clone(), env.clone())));
        self.next = next + 1;
        next
    }
    
    pub fn lookup(&self, ident : &Ident) -> Result<VClosure, SuspAt>{
        match self.map.get(ident).expect("unknown suspension ident") {
            Ok(vclos) => Ok(vclos.clone()),
            Err((comp, env)) => Err(SuspAt { ident : *ident, comp : comp.clone(), env : env.clone() })
        }
    }
    
    pub fn set(&mut self, ident : &Ident, val : &Rc<MValue>, env : &Rc<Env>) {
        self.map.insert(*ident, Ok(VClosure::mk_clos(&val, &env)));
    }
    
    pub fn next(&self) -> Option<SuspAt> {
        if let Some((ident, Err((comp, env)))) = self.map.iter().find(|(_, w)| w.is_err()) {
            Some(SuspAt { ident : *ident , comp : comp.clone(), env : env.clone() })
        }
        else { None }
    }
    
}