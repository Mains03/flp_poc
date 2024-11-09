use std::{cell::RefCell, ptr, rc::Rc};

use crate::cbpv::terms::ValueType;

use super::{env::Env, mterms::MValue, VClosure};


#[derive(Debug)]
pub struct LogicVar {
    ptype : ValueType,
    vclos : RefCell<Option<VClosure>>
}

impl LogicVar {
    pub fn new(ptype : ValueType) -> Rc<Self> {
        Rc::new(LogicVar { ptype: ptype.clone(), vclos : RefCell::new(None) })
    }
    
    pub fn get_type(&self) -> ValueType {
        self.ptype.clone()
    }
    
    pub fn get(&self) -> Option<VClosure> {
        self.vclos.borrow().clone()
    }
    
    pub fn resolved(&self) -> bool {
        self.vclos.borrow().is_some()
    }

    fn with_val_new(ptype : &ValueType, val : &Rc<MValue>, env : &Rc<Env>) -> Self {
        LogicVar {
            ptype: ptype.clone(), 
            vclos : RefCell::new(Some(VClosure::Clos { val: val.clone(), env : env.clone() }))
        }
    }
    
    pub fn set_val(&self, val : Rc<MValue>, env : Rc<Env>) {
        *(self.vclos.borrow_mut()) = Some(VClosure::Clos { val, env });
    }
    
    pub fn set_vclos(&self, vclos : &VClosure) {
        *(self.vclos.borrow_mut()) = Some(vclos.clone());
    }
    
}

impl Clone for LogicVar {
    fn clone(&self) -> LogicVar  {
        LogicVar {
            ptype : self.ptype.clone(),
            vclos : RefCell::new(self.vclos.borrow().clone())
        }
    }
}

impl PartialEq for LogicVar {
    fn eq(&self, other : &Self) -> bool {
        self.ptype == other.ptype && ptr::eq(&self.vclos, &other.vclos) 
    }
}