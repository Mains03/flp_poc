use std::cmp;

use super::term::Term;

pub fn eval_equate<'a>(lhs: Term<'a>, rhs: Term<'a>, body: Term<'a>) -> Term<'a> {
    if body == Term::Fail {
        Term::Fail
    } else if lhs == rhs {
        body
    } else if is_equate_cycle(&lhs, &rhs) {
        Term::Fail
    } else {
        if is_succ(&lhs) {
            if is_succ(&rhs) {
                let (lhs, rhs) = remove_succ(lhs, rhs).unwrap();
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

fn is_equate_cycle(lhs: &Term, rhs: &Term) -> bool {
    match lhs {
        Term::Var(s1) => match rhs {
            Term::Succ(_, t) => match t {
                Some(t) => match **t {
                    Term::Var(ref s2) => s1 == s2,
                    _ => false
                },
                None => false,
            },
            _ => false
        },
        Term::Succ(_, t) => match t {
            Some(t) => match **t {
                Term::Var(ref s1) => match rhs {
                    Term::Var(s2) => s1 == s2,
                    _ => false
                },
                _ => false
            },
            None => false
        },
        _ => false
    }
}

fn is_succ(term: &Term) -> bool {
    match term {
        Term::Succ(n, _) => *n != 0,
        _ => false
    }
}

fn is_zero(term: &Term) -> bool {
    match term {
        Term::Succ(n, v) => match v {
            Some(_) => false,
            None => *n == 0
        },
        _ => false
    }
}

fn remove_succ<'a>(lhs: Term<'a>, rhs: Term<'a>) -> Option<(Term<'a>, Term<'a>)> {
    match lhs {
        Term::Succ(n1, v1) => match rhs {
            Term::Succ(n2, v2) => {
                let val = cmp::min(n1, n2);

                Some((Term::Succ(n1 - val, v1), Term::Succ(n2 - val, v2)))
            },
            _ => None
        },
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let lhs = Term::Succ(1, Some(Box::new(Term::Var("n".to_string()))));
        
        let rhs = Term::Var("n".to_string());
        
        let body = Term::Return(Box::new(Term::Succ(1, None)));

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }

    #[test]
    fn test2() {
        let lhs = Term::Var("n".to_string());

        let rhs = Term::Succ(1, None);

        let body = Term::Fail;

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }

    #[test]
    fn test3() {
        let lhs = Term::Succ(5, Some(Box::new(Term::Var("n".to_string()))));

        let rhs = Term::Succ(5, Some(Box::new(Term::Var("m".to_string()))));

        let body = Term::Succ(1, None);

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Equate {
                lhs: Box::new(Term::Succ(0, Some(Box::new(Term::Var("n".to_string()))))),
                rhs: Box::new(Term::Succ(0, Some(Box::new(Term::Var("m".to_string()))))),
                body: Box::new(Term::Succ(1, None))
            }
        );
    }

    #[test]
    fn test4() {
        let lhs = Term::Succ(5, None);

        let rhs = Term::Succ(3, None);

        let body = Term::Succ(1, None);

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Equate {
                lhs: Box::new(Term::Succ(2, None)),
                rhs: Box::new(Term::Succ(0, None)),
                body: Box::new(Term::Succ(1, None))
            }
        );
    }

    #[test]
    fn test5() {
        let lhs = Term::Succ(2, None);

        let rhs = Term::Succ(0, None);

        let body = Term::Succ(1, None);

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }
}