use std::{collections::VecDeque, rc::Rc};

use super::{env::Env, lvar::LogicEnv, mterms::MValue, VClosure};

pub fn unify(lhs : &Rc<MValue>, rhs : &Rc<MValue>, env : &Rc<Env>, mut lenv : LogicEnv) -> bool { 
    let mut q : VecDeque<(Rc<VClosure>, Rc<VClosure>)> = VecDeque::new();
    
    let lhs_clos = VClosure::Clos { val: lhs.clone(), env: env.clone() }.into();
    let rhs_clos = VClosure::Clos { val: rhs.clone(), env: env.clone() }.into();
    
    q.push_back((lhs_clos, rhs_clos));
    while let Some((lhs, rhs)) = q.pop_front() {
        let lhs = lhs.close_head(&lenv);
        let rhs = rhs.close_head(&lenv);
        match (&*lhs, &*rhs) {
            (VClosure::LogicVar { ident : ident_lhs}, VClosure::LogicVar { ident : ident_rhs}) => { 
                // both are variables! equalize them anyway
                lenv.identify(ident_lhs, ident_rhs);
            },
            (VClosure::LogicVar { ident }, _) => { 
                // the head of the LHS has been closed, so it must be a free logic variable
                if rhs.occurs_lvar(&lenv, ident) { return false }
                lenv.set_vclos(ident, &rhs);
            },
            (_, VClosure::LogicVar { ident }) => { 
                if lhs.occurs_lvar(&lenv, ident) { return false }
                lenv.set_vclos(ident, &lhs);
            },
            (VClosure::Clos { val : lhs_val, env: lhs_env}, VClosure::Clos { val : rhs_val, env : rhs_env }) =>
                match (&**lhs_val, &**rhs_val) {
                    (MValue::Zero, MValue::Zero) => { continue },
                    (MValue::Zero, _) => { return false },
                    (MValue::Succ(v), MValue::Succ(w)) => {
                        q.push_back((VClosure::Clos { val: v.clone(), env: lhs_env.clone() }.into(), VClosure::Clos { val : w.clone(), env : rhs_env.clone()}.into()));
                    }
                    (MValue::Succ(_), _) => { return false }
                    (MValue::Nil, MValue::Nil) => continue,
                    (MValue::Nil, _) => { return false },
                    (MValue::Cons(x, xs), MValue::Cons(y, ys)) => { 
                        q.push_back((VClosure::Clos { val: x.clone(), env: lhs_env.clone() }.into(), VClosure::Clos { val : y.clone(), env : rhs_env.clone()}.into()));
                        q.push_back((VClosure::Clos { val: xs.clone(), env: lhs_env.clone() }.into(), VClosure::Clos { val : ys.clone(), env : rhs_env.clone()}.into()));
                    }
                    (MValue::Cons(_, _), _) => { return false }
                    _ => continue
                }
        }
    }
    return true
} 