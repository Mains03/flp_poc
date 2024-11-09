use std::rc::Rc;

use super::{env::Env, lvar::LogicVar, mterms::MValue};

#[derive(Clone, Debug)]
pub enum VClosure {
    Clos { val : Rc<MValue>, env : Rc<Env> },
    LogicVar { lvar : Rc<LogicVar> } // by making this Rc we ensure that cloning is never deep
}

impl VClosure {
    pub fn val(&self) -> String { 
        match self {
            VClosure::Clos { val, env } => val.to_string(),
            VClosure::LogicVar { lvar } => "logic var".to_string(),
        }
    }
    
    pub fn deep_clone(self: &Rc<Self>) -> Rc<Self> {
        if self.has_unresolved_lvars() {
            match &**self {
                Self::Clos { val, env } => Self::Clos { val: val.clone(), env: env.deep_clone() },
                Self::LogicVar { lvar } => Self::LogicVar { lvar: Rc::new((**lvar).clone()) },
            }.into()
        }
        else {
            self.clone()
        }
    }
    
    pub fn has_unresolved_lvars(&self) -> bool {
        match self {
            Self::Clos { val, env } => env.has_unresolved_lvars(),
            Self::LogicVar { lvar } => !lvar.resolved()
        }
    }

    pub fn occurs_lvar(&self, lvar : &LogicVar) -> bool {
        let vclos = self.close_head();
        match &*vclos {
            VClosure::Clos { val, env } => {
                match &**val {
                    MValue::Succ(v) => VClosure::Clos {val : v.clone(), env: env.clone() }.occurs_lvar(lvar),
                    MValue::Cons(v, w) => 
                        VClosure::Clos { val : v.clone(), env: env.clone()}.occurs_lvar(lvar)
                        || VClosure::Clos { val : w.clone(), env : env.clone()}.occurs_lvar(lvar),
                    MValue::Var(_) => unreachable!("value should be head-closed in occurs check"),
                    MValue::Thunk(_) => panic!("mustn't be doing occurs to a computation"),
                    _ => false
                }
            },
            VClosure::LogicVar { lvar: _lvar } => *lvar == **_lvar,
        }
    }

    pub fn close_head(&self) -> Rc<VClosure> {
        let mut vclos = self.clone();
        loop {
            vclos = match vclos {
                VClosure::Clos { ref val, ref env } => {
                    match **val {
                        MValue::Var(i) => env.lookup(i).clone(),
                        _ => break
                    }
                },
                VClosure::LogicVar { ref lvar } => {
                    match lvar.get() {
                        Some(v) => v,
                        None => break,
                    }
                }
            }
        }
        Rc::new(vclos)
    }

    pub fn close_val(&self) -> MValue {
        match self {
            VClosure::Clos { val,  env } => {
                // println!("[DEBUG] CLOSING {} in env of size {}: {:#?}", val, env.len(), *env);
                match &**val {
                    MValue::Var(i) => env.lookup(*i).close_val(),
                    MValue::Zero => MValue::Zero,
                    MValue::Succ(v) => MValue::Succ(VClosure::Clos { val: v.clone(), env : env.clone() }.close_val().into()),
                    MValue::Nil => MValue::Nil,
                    MValue::Cons(v, w) => 
                        MValue::Cons(
                            VClosure::Clos { val: v.clone(), env : env.clone() }.close_val().into(),
                            VClosure::Clos { val: w.clone(), env : env.clone() }.close_val().into()
                        ),
                    MValue::Pair(fst, snd) => 
                        MValue::Pair(
                            VClosure::Clos{ val : fst.clone(), env : env.clone()}.close_val().into(),
                            VClosure::Clos{ val : snd.clone(), env : env.clone() }.close_val().into(),
                        ),
                    MValue::Inl(v) =>
                        MValue::Inl(VClosure::Clos{ val : v.clone(), env : env.clone() }.close_val().into()),
                    MValue::Inr(v) => 
                        MValue::Inr(VClosure::Clos{ val : v.clone(), env : env.clone() }.close_val().into()),
                    MValue::Thunk(t) => panic!("shouldn't be returning a thunk anyway: {}", *t),
                }
            },
            VClosure::LogicVar { ref lvar } => {
                match lvar.get() {
                    Some(v) => v.close_val(),
                    None => panic!("unresolved logic var"),
                }
            }
        }
    }

}
