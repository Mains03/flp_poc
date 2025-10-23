use std::{collections::VecDeque, rc::Rc};

use super::{env::Env, lvar::LogicEnv, mterms::MValue, senv::SuspEnv, senv::SuspAt, VClosure};

pub enum UnifyError {
    Occurs,
    Fail,
    Susp(SuspAt),
}

pub fn unify(lhs : &Rc<MValue>, rhs : &Rc<MValue>, env : &Rc<Env>, lenv : &mut LogicEnv, senv : &SuspEnv) -> Result<(),UnifyError> { 

    let mut q : VecDeque<(VClosure, VClosure)> = VecDeque::new();
    let lhs_clos = VClosure::mk_clos(lhs, env);
    let rhs_clos = VClosure::mk_clos(rhs, env);
    q.push_back((lhs_clos, rhs_clos));

    while let Some((lhs, rhs)) = q.pop_front() {

        // close the LHS and RHS to find what their head is
        let lhs = lhs.close_head(&lenv, senv).map_err(UnifyError::Susp)?;
        let rhs = rhs.close_head(&lenv, senv).map_err(UnifyError::Susp)?;

        // println!("[DEBUG] about to unify {} and {}", lhs.val(), rhs.val());
        match (&lhs, &rhs) {
            (VClosure::LogicVar { ident : ident_lhs}, VClosure::LogicVar { ident : ident_rhs}) => { 
                // both are variables, so make them equal in the logic env
                lenv.identify(*ident_lhs, *ident_rhs);
            },
            (VClosure::LogicVar { ident }, _) => { 
                // the LHS is a logic variable
                if rhs.occurs_lvar(&lenv, senv, *ident).map_err(UnifyError::Susp)? { return Err(UnifyError::Occurs) }
                lenv.set_vclos(*ident, rhs);
            },
            (_, VClosure::LogicVar { ident }) => { 
                // the RHS is a logic variable
                if lhs.occurs_lvar(&lenv, senv, *ident).map_err(UnifyError::Susp)? { return Err(UnifyError::Occurs) }
                lenv.set_vclos(*ident, lhs);
            },
            (VClosure::Clos { val : lhs_val, env: lhs_env}, VClosure::Clos { val : rhs_val, env : rhs_env }) =>
                match (&**lhs_val, &**rhs_val) {
                    (MValue::Zero, MValue::Zero) => continue,
                    (MValue::Zero, _) => { return Err(UnifyError::Fail) },
                    (MValue::Succ(v), MValue::Succ(w)) => {
                        q.push_back((VClosure::mk_clos(v, &lhs_env), VClosure::mk_clos(w, &rhs_env)));
                    }
                    (MValue::Succ(_), _) => { return Err(UnifyError::Fail) }
                    (MValue::Nil, MValue::Nil) => continue,
                    (MValue::Nil, _) => { return Err(UnifyError::Fail) },
                    (MValue::Cons(x, xs), MValue::Cons(y, ys)) => { 
                        q.push_back((VClosure::mk_clos(x, &lhs_env), VClosure::mk_clos(y, &rhs_env)));
                        q.push_back((VClosure::mk_clos(xs, &lhs_env), VClosure::mk_clos(ys, &rhs_env)));
                    }
                    (MValue::Cons(_, _), _) => { return Err(UnifyError::Fail) }
                    _ => { panic!("tried to unify a thunk") }
                }
            (VClosure::Susp { ident }, _) => unreachable!("tried to unify a suspension"),
            (_, VClosure::Susp { ident }) => unreachable!("tried to unify a suspension"),
        }
    }
    return Ok(())
} 
