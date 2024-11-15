use std::rc::Rc;

use super::{env::Env, lvar::LogicEnv, mterms::MValue, Ident};

#[derive(Clone, PartialEq, Debug)]
pub enum VClosure {
    Clos { val : Rc<MValue>, env : Rc<Env> },
    LogicVar { ident : Ident }
}

impl VClosure {
    pub fn val(&self) -> String { 
        match self {
            VClosure::Clos { val, env } => format!("value: {}", val.to_string()),
            VClosure::LogicVar { ident } => format!("logic var: {}", ident)
        }
    }
    
    pub fn occurs_lvar(&self, lenv : &LogicEnv, ident : &Ident) -> bool {
        let vclos = self.close_head(lenv);
        match &*vclos {
            VClosure::Clos { val, env } => {
                match &**val {
                    MValue::Succ(v) => VClosure::Clos {val : v.clone(), env: env.clone() }.occurs_lvar(lenv, ident),
                    MValue::Cons(v, w) => 
                        VClosure::Clos { val : v.clone(), env: env.clone()}.occurs_lvar(lenv, ident)
                        || VClosure::Clos { val : w.clone(), env : env.clone()}.occurs_lvar(lenv, ident),
                    MValue::Var(_) => unreachable!("value should be head-closed in occurs check"),
                    MValue::Thunk(_) => panic!("mustn't be doing occurs to a computation"),
                    _ => false
                }
            },
            VClosure::LogicVar { ident } => lenv.lookup(ident).expect("oops").occurs_lvar(lenv, ident) , // *lvar == **_lvar,
        }
    }

    pub fn close_head(&self, lenv : &LogicEnv) -> Rc<VClosure> {
        let mut vclos = self.clone();
        loop {
            vclos = match vclos {
                VClosure::Clos { ref val, ref env } => {
                    match **val {
                        MValue::Var(i) => env.lookup(i).expect("failed to find index").clone(),
                        _ => break
                    }
                },
                VClosure::LogicVar { ref ident } => {
                    match lenv.lookup(ident) {
                        Some(vclos) => (*vclos).clone(),
                        None => break,
                    }
                }
            }
        }
        vclos.into()
    }

    pub fn close_head_err(&self, lenv : &LogicEnv) -> Result<Rc<VClosure>, ()> {
        let mut vclos = self.clone();
        loop {
            vclos = match vclos {
                VClosure::Clos { ref val, ref env } => {
                    match **val {
                        MValue::Var(i) => {
                            match env.lookup(i) {
                                Some(vclos) => vclos.clone(),
                                None => return Err(()),
                            }
                        },
                        _ => break
                    }
                },
                VClosure::LogicVar { ref ident } => {
                    match lenv.lookup(ident) {
                        Some(vclos) => (*vclos).clone(),
                        None => break,
                    }
                }
            }
        }
        Ok(vclos.into())
    }

    pub fn close_val(&self, lenv : &LogicEnv) -> Option<MValue> {
        match self {
            VClosure::Clos { val,  env } => {
                // println!("[DEBUG] CLOSING {:?} in env of size {}", val, env.size());
                match &**val {
                    MValue::Var(i) => env.lookup(*i)?.close_val(lenv),
                    MValue::Zero => Some(MValue::Zero),
                    MValue::Succ(v) => Some(MValue::Succ(VClosure::Clos { val: v.clone(), env : env.clone() }.close_val(lenv)?.into())),
                    MValue::Nil => Some(MValue::Nil),
                    MValue::Cons(v, w) => 
                        Some(MValue::Cons(
                            VClosure::Clos { val: v.clone(), env : env.clone() }.close_val(lenv)?.into(),
                            VClosure::Clos { val: w.clone(), env : env.clone() }.close_val(lenv)?.into()
                        )),
                    MValue::Pair(fst, snd) => 
                        Some(MValue::Pair(
                            VClosure::Clos{ val : fst.clone(), env : env.clone()}.close_val(lenv)?.into(),
                            VClosure::Clos{ val : snd.clone(), env : env.clone() }.close_val(lenv)?.into(),
                        )),
                    MValue::Inl(v) =>
                        Some(MValue::Inl(VClosure::Clos{ val : v.clone(), env : env.clone() }.close_val(lenv)?.into())),
                    MValue::Inr(v) => 
                        Some(MValue::Inr(VClosure::Clos{ val : v.clone(), env : env.clone() }.close_val(lenv)?.into())),
                    MValue::Thunk(t) => panic!("shouldn't be returning a thunk anyway: {}", *t),
                }
            },
            VClosure::LogicVar { ref ident } => {
                match lenv.lookup(ident) {
                    Some(v) => v.close_val(lenv),
                    None => None,
                }
            }
        }
    }

}
