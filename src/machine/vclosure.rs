use std::rc::Rc;

use crate::machine::senv::SuspAt;

use super::{env::Env, lvar::LogicEnv, mterms::{MComputation, MValue}, senv::SuspEnv, Ident};

#[derive(Clone, Debug)]
pub enum VClosure {
    Clos { val : Rc<MValue>, env : Rc<Env> },
    LogicVar { ident : Ident },
    Susp { ident : Ident }
}

impl VClosure {

    pub fn val(&self) -> String { 
        match self {
            VClosure::Clos { val, env } => format!("value: {}", val.to_string()),
            VClosure::LogicVar { ident } => format!("logic var: {}", ident),
            VClosure::Susp { ident } => format!("suspension: {}", ident),
        }
    }

    pub fn mk_clos(val : &Rc<MValue>, env : &Rc<Env>) -> VClosure {
        VClosure::Clos { val : val.clone(), env : env.clone() }
    }

    pub fn occurs_lvar(&self, lenv : &LogicEnv, senv : &SuspEnv, ident : Ident) -> Result<bool, SuspAt> {
        match self.clone().close_head(lenv, &senv)? {
            VClosure::Clos { val, env } => {
                match &*val {
                    MValue::Succ(v) => VClosure::mk_clos(v, &env).occurs_lvar(lenv, senv, ident),
                    MValue::Cons(v, w) => 
                        Ok(VClosure::Clos { val : v.clone(), env: env.clone()}.occurs_lvar(lenv, senv, ident)?
                        || VClosure::Clos { val : w.clone(), env : env.clone()}.occurs_lvar(lenv, senv, ident)?),
                    MValue::Var(_) => unreachable!("value should be head-closed in occurs check"),
                    MValue::Thunk(_) => panic!("mustn't be doing occurs to a computation"),
                    _ => Ok(false)
                }
            },
            VClosure::LogicVar { ident : ident2 } => Ok(ident == ident2),
            VClosure::Susp { ident } => todo!()
        }
    }
    
    pub fn find_susp(self, lenv : &LogicEnv, senv : &SuspEnv) -> Option<SuspAt> {
        match self.close_head(lenv, senv) {
            Ok(VClosure::Clos { val, env }) => 
                match &*val {
                    MValue::Var(i) => env.lookup(*i).expect("oops").find_susp(lenv, senv),
                    MValue::Zero => None,
                    MValue::Succ(v) => VClosure::mk_clos(v, &env).find_susp(lenv, senv),
                    MValue::Pair(v, w) => 
                        VClosure::mk_clos(v, &env).find_susp(lenv, senv)
                        .or(VClosure::mk_clos(w, &env).find_susp(lenv, senv)),
                    MValue::Inl(v) => VClosure::mk_clos(v, &env).find_susp(lenv, senv),
                    MValue::Inr(v) => VClosure::mk_clos(v, &env).find_susp(lenv, senv),
                    MValue::Nil => None,
                    MValue::Cons(v, w) =>
                        VClosure::mk_clos(v, &env).find_susp(lenv, senv)
                        .or(VClosure::mk_clos(w, &env).find_susp(lenv, senv)),
                    MValue::Thunk(t) => None,
                },
            Ok(VClosure::LogicVar { ident }) => lenv.lookup(ident)?.find_susp(lenv, senv),
            Ok(VClosure::Susp { .. }) => unreachable!("there is an unexpected suspension"),
            Err(a) => Some(a)
        }
    }

    pub fn close_head(self, lenv : &LogicEnv, senv : &SuspEnv) -> Result<VClosure, SuspAt> {
        let mut vclos = self;
        loop {
            vclos = match &vclos {
                VClosure::Clos { val, env } => {
                    match &**val {
                        MValue::Var(i) => env.lookup(*i).expect("failed to find index"),
                        _ => break
                    }
                },
                VClosure::LogicVar { ident } => {
                    match lenv.lookup(*ident) {
                        Some(vclos) => vclos,
                        None => break,
                    }
                }
                VClosure::Susp { ident } => senv.lookup(ident)?
            }
        }
        Ok(vclos)
    }

    pub fn close_val(&self, lenv : &LogicEnv, senv : &SuspEnv) -> Option<MValue> {
        match self {
            VClosure::Clos { val,  env } => {
                // println!("[DEBUG] CLOSING {:?} in env of size {}", val, env.size());
                match &**val {
                    MValue::Var(i) => env.lookup(*i)?.close_val(lenv, senv),
                    MValue::Zero => Some(MValue::Zero),
                    MValue::Succ(v) => Some(MValue::Succ(VClosure::mk_clos(v, env).close_val(lenv, senv)?.into())),
                    MValue::Nil => Some(MValue::Nil),
                    MValue::Cons(v, w) => 
                        Some(MValue::Cons(
                            VClosure::mk_clos(v, env).close_val(lenv, senv)?.into(),
                            VClosure::mk_clos(w, env).close_val(lenv, senv)?.into()
                        )),
                    MValue::Pair(fst, snd) => 
                        Some(MValue::Pair(
                            VClosure::mk_clos(fst, env).close_val(lenv, senv)?.into(),
                            VClosure::mk_clos(snd, env).close_val(lenv, senv)?.into(),
                        )),
                    MValue::Inl(v) =>
                        Some(MValue::Inl(VClosure::mk_clos(v, env).close_val(lenv, senv)?.into())),
                    MValue::Inr(v) => 
                        Some(MValue::Inr(VClosure::mk_clos(v, env).close_val(lenv, senv)?.into())),
                    MValue::Thunk(t) => panic!("tried to close thunk: {}", *t),
                }
            },
            VClosure::LogicVar { ref ident } => {
                match lenv.lookup(*ident) {
                    Some(v) => v.close_val(lenv, senv),
                    None => None,
                }
            }
            VClosure::Susp { ident } => {
                match senv.lookup(ident) {
                    Ok(vclos) => vclos.close_val(lenv, senv),
                    Err(_) => panic!("trying to close value with unresolved suspension"),
                }
            },
        }
    }

}
