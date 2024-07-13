use std::cmp;

use super::term::{get_var, is_var, Term};

pub fn eval_equate<'a>(lhs: Term<'a>, rhs: Term<'a>, body: Term<'a>) -> Term<'a> {
    if body == Term::Fail {
        Term::Fail
    }  else if is_eq_equate_cycle(&lhs, &rhs) {
        Term::Fail
    } else {
        if is_succ(&lhs) {
            if is_succ(&rhs) {
                let (lhs, rhs) = remove_succ(lhs, rhs);
                Term::Equate { lhs: Box::new(lhs), rhs: Box::new(rhs), body: Box::new(body) }
            } else if is_zero(&rhs) {
                Term::Fail
            } else {
                Term::Equate { lhs: Box::new(lhs), rhs: Box::new(rhs), body: Box::new(body) }
            }
        } else if is_succ(&rhs) {
            if is_zero(&lhs) {
                Term::Fail
            } else {
                Term::Equate { lhs: Box::new(lhs), rhs: Box::new(rhs), body: Box::new(body) }
            }
        } else {
            Term::Equate { lhs: Box::new(lhs), rhs: Box::new(rhs), body: Box::new(body) }
        }
    }
}

fn is_eq_equate_cycle(lhs: &Term, rhs: &Term) -> bool {
    if is_var(lhs) {
        let var = get_var(lhs);

        if is_succ_of(rhs, &var) {
            true
        } else {
            false
        }
    } else if is_var(rhs) {
        let var = get_var(rhs);

        if is_succ_of(lhs, &var) {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn is_succ(term: &Term) -> bool {
    match term {
        Term::Nat(n) => *n != 0,
        Term::Add(lhs, rhs) => is_succ(&*lhs) || is_succ(&*rhs),
        _ => false
    }
}

fn is_succ_of(term: &Term, var: &str) -> bool {
    match term {
        Term::Add(lhs, rhs) => {
            let lhs_flag = contains_one_instance_of(&*lhs, var);
            let rhs_flag = contains_one_instance_of(&*rhs, var);
            
            if lhs_flag ^ rhs_flag {
                if lhs_flag {
                    is_succ_of(&*lhs, var) || !is_zero(&*rhs)
                } else {
                    is_succ_of(&*rhs, var) || !is_zero(&*lhs)
                }
            } else {
                false
            }
        },
        _ => false
    }
}

fn contains_one_instance_of(term: &Term, var: &str) -> bool {
    match term {
        Term::Var(s) => s == var,
        Term::Nat(_) => false,
        Term::Add(lhs, rhs) => {
            let var_count = var_count(&*lhs) + var_count(&*rhs);

            if var_count == 0 || var_count > 1 {
                false
            } else {
                contains_one_instance_of(&*&lhs, var) || contains_one_instance_of(&*rhs, var)
            }
        },
        _ => false
    }
}

fn var_count(term: &Term) -> usize {
    match term {
        Term::Var(_) => 1,
        Term::Add(lhs, rhs) => var_count(&*lhs) + var_count(&*rhs),
        _ => 0
    }
}

fn is_zero(term: &Term) -> bool {
    match term {
        Term::Nat(n) => *n == 0,
        Term::Add(lhs, rhs) => is_zero(&*lhs) && is_zero(&*rhs),
        _ => false
    }
}

fn remove_succ<'a>(lhs: Term<'a>, rhs: Term<'a>) -> (Term<'a>, Term<'a>) {
    let lhs_val = get_value(&lhs);
    let rhs_val = get_value(&rhs);

    let val = cmp::min(lhs_val, rhs_val);
    
    (subtract(lhs, val).0, subtract(rhs, val).0)
}

fn get_value(term: &Term) -> usize {
    match term {
        Term::Nat(n) => *n,
        Term::Add(lhs, rhs) => get_value(&*lhs) + get_value(&*rhs),
        _ => 0
    }
}

fn subtract<'a>(term: Term<'a>, val: usize) -> (Term<'a>, usize) {
    match term {
        Term::Nat(n) => {
            let remainder = val - cmp::min(n, val);

            (Term::Nat(n - (val - remainder)), remainder)
        },
        Term::Add(lhs, rhs) => {
            let (lhs, val) = subtract(*lhs, val);
            let (rhs, val) = subtract(*rhs, val);

            (Term::Add(Box::new(lhs), Box::new(rhs)), val)
        },
        t => (t, val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let lhs = Term::Add(
                Box::new(Term::Add(
                    Box::new(Term::Var("n".to_string())),
                    Box::new(Term::Nat(1))
                )),
                Box::new(Term::Nat(0))
            );
        
        let rhs = Term::Var("n".to_string());
        
        let body = Term::Return(Box::new(Term::Nat(1)));

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }

    #[test]
    fn test2() {
        let lhs = Term::Add(
            Box::new(Term::Var("n".to_string())),
            Box::new(Term::Add(
                Box::new(Term::Add(
                    Box::new(Term::Add(
                        Box::new(Term::Var("m".to_string())),
                        Box::new(Term::Nat(3))
                    )),
                    Box::new(Term::Nat(2))
                )),
                Box::new(Term::Nat(1))
            ))
        );

        let rhs = Term::Add(
            Box::new(Term::Nat(3)),
            Box::new(Term::Nat(2))
        );

        let body = Term::Return(Box::new(Term::Nat(1)));

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Equate {
                lhs: Box::new(Term::Add(
                    Box::new(Term::Var("n".to_string())),
                    Box::new(Term::Add(
                        Box::new(Term::Add(
                            Box::new(Term::Add(
                                Box::new(Term::Var("m".to_string())),
                                Box::new(Term::Nat(0))
                            )),
                            Box::new(Term::Nat(0))
                        )),
                        Box::new(Term::Nat(1))
                    ))
                )),
                rhs: Box::new(Term::Add(
                    Box::new(Term::Nat(0)),
                    Box::new(Term::Nat(0))
                )),
                body: Box::new(Term::Return(Box::new(Term::Nat(1))))
            }
        );
    }

    #[test]
    fn test3() {
        let lhs = Term::Add(
            Box::new(Term::Var("n".to_string())),
            Box::new(Term::Add(
                Box::new(Term::Add(
                    Box::new(Term::Add(
                        Box::new(Term::Var("m".to_string())),
                        Box::new(Term::Nat(0))
                    )),
                    Box::new(Term::Nat(0))
                )),
                Box::new(Term::Nat(1))
            ))
        );
          
        let rhs = Term::Add(
            Box::new(Term::Nat(0)),
            Box::new(Term::Nat(0))
        );

        let body = Term::Return(Box::new(Term::Nat(1)));
        
        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }

    #[test]
    fn test4() {
        let lhs = Term::Add(
            Box::new(Term::Var("n".to_string())),
            Box::new(Term::Add(
                Box::new(Term::Add(
                    Box::new(Term::Add(
                        Box::new(Term::Var("m".to_string())),
                        Box::new(Term::Nat(0))
                    )),
                    Box::new(Term::Nat(0))
                )),
                Box::new(Term::Nat(0))
            ))
        );
          
        let rhs = Term::Add(
            Box::new(Term::Nat(0)),
            Box::new(Term::Nat(0))
        );

        let body = Term::Return(Box::new(Term::Nat(1)));
        
        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Equate {
                lhs: Box::new(Term::Add(
                    Box::new(Term::Var("n".to_string())),
                    Box::new(Term::Add(
                        Box::new(Term::Add(
                            Box::new(Term::Add(
                                Box::new(Term::Var("m".to_string())),
                                Box::new(Term::Nat(0))
                            )),
                            Box::new(Term::Nat(0))
                        )),
                        Box::new(Term::Nat(0))
                    )))),
                rhs: Box::new(Term::Add(
                    Box::new(Term::Nat(0)),
                    Box::new(Term::Nat(0))
                )),
                body: Box::new(Term::Return(Box::new(Term::Nat(1))))
            }
        );
    }

    #[test]
    fn test5() {
        let lhs = Term::Var("n".to_string());

        let rhs = Term::Nat(1);

        let body = Term::Fail;

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }
}