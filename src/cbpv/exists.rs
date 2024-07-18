use std::collections::HashMap;

use crate::parser::syntax::r#type::Type;

use super::term::Term;

pub fn eval_exists<'a>(var: &'a str, r#type: Type<'a>, term: Term<'a>, env: &HashMap<String, Term<'a>>) -> Term<'a> {
    if term == Term::Fail {
        Term::Fail
    } else {
        match term.eval(env) {
            Term::Equate { lhs, rhs, body } => {
                if lhs == rhs {
                    *body
                } else if is_bullseye(var, &*lhs, &*rhs) {
                    substitute_bullseye(*lhs, *rhs, *body).unwrap()
                } else {
                    enumerate(var, r#type, Term::Equate { lhs, rhs, body })
                }
            },
            Term::Fail => Term::Fail,
            term => enumerate(var, r#type, term)
        }
    }
}

fn is_bullseye(var: &str, lhs: &Term, rhs: &Term) -> bool {
    match lhs {
        Term::Var(v) => v == var,
        _ => match rhs {
            Term::Var(v) => v == var,
            _ => false
        }
    }
}

fn substitute_bullseye<'a>(lhs: Term<'a>, rhs: Term<'a>, body: Term<'a>) -> Option<Term<'a>> {
    match lhs {
        Term::Var(v) => Some(body.substitute(&v, &rhs)),
        _ => match rhs {
            Term::Var(v) => Some(body.substitute(&v, &lhs)),
            _ => None
        }
    }
}

fn enumerate<'a>(var: &'a str, r#type: Type<'a>, term: Term<'a>) -> Term<'a> {
    match r#type {
        Type::Ident(t) => if t == "Nat" {
            Term::Choice(vec![
                term.clone().substitute(var, &Term::Zero),
                Term::Exists {
                    var,
                    r#type,
                    body: Box::new(term.substitute(var, 
                            &Term::Succ(Box::new(Term::Var(var.to_string())))
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
            body: Box::new(Term::Return(Box::new(Term::Zero)))
        };

        let term = eval_exists(var, r#type, term, &HashMap::new());

        assert_eq!(
            term,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Zero)),
                Term::Exists { var: "n", r#type: Type::Ident("Nat"), body: Box::new(Term::Return(Box::new(Term::Zero))) }
            ])
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

        let term = eval_exists(var, r#type, term, &HashMap::new());

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
            rhs: Box::new(Term::Succ(Box::new(Term::Zero))),
            body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
        };

        let term = eval_exists(var, r#type, term, &HashMap::new());

        assert_eq!(
            term,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))
        );
    }

    #[test]
    fn test4() {
        let var = "n";

        let r#type = Type::Ident("Nat");

        let term = Term::Return(Box::new(Term::Var("n".to_string())));

        let term = eval_exists(var, r#type, term, &HashMap::new());

        assert_eq!(
            term,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Zero)),
                Term::Exists {
                    var: "n",
                    r#type: Type::Ident("Nat"),
                    body: Box::new(Term::Return(Box::new(
                        Term::Succ(Box::new(Term::Var("n".to_string()))))
                    ))
                }
            ])
        );
    }
}