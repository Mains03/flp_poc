use crate::parser::syntax::r#type::Type;

use super::term::{substitute, Term};

pub fn eval_exists<'a>(var: &'a str, r#type: Type<'a>, term: Term<'a>) -> Term<'a> {
    if term == Term::Fail {
        Term::Fail
    } else {
        match term {
            Term::Equate { lhs, rhs, body } => {
                if lhs == rhs {
                    *body
                } else if is_bullseye(var, &r#type, &*lhs, &*rhs) {
                    substitute_bullseye(*lhs, *rhs, *body).unwrap()
                } else if is_value(&*lhs, &r#type) && is_value(&*rhs, &r#type) {
                    Term::Fail // already checked for equality
                } else {
                    enumerate(var, r#type, Term::Equate { lhs, rhs, body })
                }
            },
            _ => enumerate(var, r#type, term)
        }
    }
}

fn is_bullseye(var: &str, r#type: &Type, lhs: &Term, rhs: &Term) -> bool {
    match lhs {
        Term::Var(v) => v == var && is_value(rhs, r#type),
        _ => match rhs {
            Term::Var(v) => v == var && is_value(lhs, r#type),
            _ => false
        }
    }
}

fn substitute_bullseye<'a>(lhs: Term<'a>, rhs: Term<'a>, body: Term<'a>) -> Option<Term<'a>> {
    match lhs {
        Term::Var(v) => Some(substitute(body, &v, &rhs)),
        _ => match rhs {
            Term::Var(v) => Some(substitute(body, &v, &lhs)),
            _ => None
        }
    }
}

fn is_value(term: &Term, r#type: &Type) -> bool {
    match r#type {
        Type::Ident(t) => if *t == "Nat" {
            match term {
                Term::Succ(_, v) => match v {
                    Some(_) => false,
                    None => true
                },
                _ => false
            }
        } else {
            unimplemented!()
        },
        _ => unimplemented!()
    }
}

fn enumerate<'a>(var: &'a str, r#type: Type<'a>, term: Term<'a>) -> Term<'a> {
    match r#type {
        Type::Ident(t) => if t == "Nat" {
            Term::Choice(vec![
                substitute(term.clone(), var, &Term::Succ(0, None)),
                Term::Exists {
                    var,
                    r#type,
                    body: Box::new(substitute(term, var, 
                            &Term::Succ(1, Some(Box::new(Term::Var(var.to_string()))))
                        ))
                }
            ])
        } else {
            unimplemented!()
        },
        _ => unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let var = "n";

        let r#type = Type::Ident("Nat");

        let term = Term::Equate {
            lhs: Box::new(Term::Var("n".to_string())),
            rhs: Box::new(Term::Var("n".to_string())),
            body: Box::new(Term::Return(Box::new(Term::Succ(1, None))))
        };

        let term = eval_exists(var, r#type, term);

        assert_eq!(
            term,
            Term::Return(Box::new(Term::Succ(1, None)))
        );
    }

    #[test]
    fn test2() {
        let var = "n";

        let r#type = Type::Ident("Nat");

        let term = Term::Equate {
            lhs: Box::new(Term::Var("n".to_string())),
            rhs: Box::new(Term::Var("n".to_string())),
            body: Box::new(Term::Fail)
        };

        let term = eval_exists(var, r#type, term);

        assert_eq!(
            term,
            Term::Fail
        );
    }

    #[test]
    fn test3() {
        let var = "n";

        let r#type = Type::Ident("Nat");

        let term = Term::Equate {
            lhs: Box::new(Term::Var("n".to_string())),
            rhs: Box::new(Term::Succ(1, None)),
            body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
        };

        let term = eval_exists(var, r#type, term);

        assert_eq!(
            term,
            Term::Return(Box::new(Term::Succ(1, None)))
        );
    }

    #[test]
    fn test4() {
        let var = "n";

        let r#type = Type::Ident("Nat");

        let term = Term::Return(Box::new(Term::Var("n".to_string())));

        let term = eval_exists(var, r#type, term);

        assert_eq!(
            term,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Succ(0, None))),
                Term::Exists {
                    var: "n",
                    r#type: Type::Ident("Nat"),
                    body: Box::new(Term::Return(Box::new(
                        Term::Succ(1, Some(Box::new(Term::Var("n".to_string()))))
                    )))
                }
            ])
        );
    }
}