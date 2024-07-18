use super::term::Term;

pub fn eval_equate<'a>(lhs: Term<'a>, rhs: Term<'a>, body: Term<'a>) -> Term<'a> {
    if body == Term::Fail {
        Term::Fail
    } else if lhs == rhs {
        body
    } else if is_equate_cycle(&lhs, &rhs) {
        Term::Fail
    } else {
        match &lhs {
            Term::Zero => match &rhs {
                Term::Succ(_) => Term::Fail,
                _ => Term::Equate { lhs: Box::new(lhs), rhs: Box::new(rhs), body: Box::new(body) }
            },
            Term::Succ(t1) => match &rhs {
                Term::Zero => Term::Fail,
                Term::Succ(t2) => Term::Equate { lhs: t1.clone(), rhs: t2.clone(), body: Box::new(body) },
                _ => Term::Equate { lhs: Box::new(lhs), rhs: Box::new(rhs), body: Box::new(body) }
            },
            _ => Term::Equate { lhs: Box::new(lhs), rhs: Box::new(rhs), body: Box::new(body) }
        }
    }
}

fn is_equate_cycle(lhs: &Term, rhs: &Term) -> bool {
    match lhs {
        Term::Var(v) => rhs.is_succ_of(&v),
        _ => match rhs {
            Term::Var(v) => lhs.is_succ_of(&v),
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let lhs = Term::Succ(Box::new(Term::Var("n".to_string())));
        
        let rhs = Term::Var("n".to_string());
        
        let body = Term::Return(Box::new(Term::Zero));

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }

    #[test]
    fn test2() {
        let lhs = Term::Var("n".to_string());

        let rhs = Term::Succ(Box::new(Term::Zero));

        let body = Term::Fail;

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }

    #[test]
    fn test3() {
        let lhs = Term::Succ(Box::new(Term::Var("n".to_string())));

        let rhs = Term::Succ(Box::new(Term::Var("m".to_string())));

        let body = Term::Zero;

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Equate {
                lhs: Box::new(Term::Var("n".to_string())),
                rhs: Box::new(Term::Var("m".to_string())),
                body: Box::new(Term::Zero)
            }
        );
    }

    #[test]
    fn test4() {
        let lhs = Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero))));

        let rhs = Term::Succ(Box::new(Term::Zero));

        let body = Term::Zero;

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Equate {
                lhs: Box::new(Term::Succ(Box::new(Term::Zero))),
                rhs: Box::new(Term::Zero),
                body: Box::new(Term::Zero)
            }
        );
    }

    #[test]
    fn test5() {
        let lhs = Term::Succ(Box::new(Term::Zero));

        let rhs = Term::Zero;

        let body = Term::Zero;

        let term = eval_equate(lhs, rhs, body);

        assert_eq!(
            term,
            Term::Fail
        );
    }
}