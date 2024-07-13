use crate::parser::syntax::r#type::Type;

use super::term::{get_var, is_var, substitute, Term};

pub fn eval_exists<'a>(var: &'a str, r#type: Type, term: Term<'a>) -> Term<'a> {
    if term == Term::Fail {
        Term::Fail
    } else {
        match r#type {
            Type::Ident(s) => {
                if s == "Nat" {
                    eval_exists_nat(var, term)
                } else {
                    unimplemented!()
                }
            },
            _ => unreachable!()
        }
    }
}

fn eval_exists_nat<'a>(var: &'a str, term: Term<'a>) -> Term<'a> {
    if is_equate_equal(&term) {
        get_equate_body(term)
    } else if is_bullseye(var, &term) {
        bullseye(var, term)
    } else {
        enumerate_nat(var, term)
    }
}

fn is_equate_equal(term: &Term) -> bool {
    match term {
        Term::Equate { lhs, rhs, body: _ } => lhs == rhs,
        _ => false
    }
}

fn get_equate_body<'a>(term: Term<'a>) -> Term<'a> {
    match term {
        Term::Equate { lhs: _, rhs: _, body } => *body,
        _ => unreachable!()
    }
}

fn is_bullseye(var: &str, term: &Term) -> bool {
    match term {
        Term::Equate { lhs, rhs, body: _ } => {
            if is_var(&*lhs) {
                let s = get_var(&*lhs);

                s == var
            } else if is_var(&*rhs) {
                let s = get_var(&*rhs);

                s == var
            } else {
                false
            }
        },
        _ => false
    }
}

fn bullseye<'a>(var: &str, term: Term<'a>) -> Term<'a> {
    match term {
        Term::Equate { lhs, rhs, body } => {
            if is_var(&*lhs) {
                substitute(*body, var, &*rhs)
            } else {
                substitute(*body, var, &*lhs)
            }
        },
        _ => unreachable!()
    }
}

fn enumerate_nat<'a>(var: &'a str, term: Term<'a>) -> Term<'a> {
    Term::Choice(vec![
        substitute(term.clone(), var, &Term::Nat(0)),
        Term::Exists {
            var,
            r#type: Type::Ident("Nat"),
            body: Box::new(substitute(term, var, &Term::Add(
                Box::new(Term::Var(var.to_string())),
                Box::new(Term::Nat(1))
            )))
        }
    ])
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
            body: Box::new(Term::Return(Box::new(Term::Nat(1))))
        };

        let term = eval_exists(var, r#type, term);

        assert_eq!(
            term,
            Term::Return(Box::new(Term::Nat(1)))
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
            rhs: Box::new(Term::Nat(1)),
            body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
        };

        let term = eval_exists(var, r#type, term);

        assert_eq!(
            term,
            Term::Return(Box::new(Term::Nat(1)))
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
                Term::Return(Box::new(Term::Nat(0))),
                Term::Exists {
                    var: "n",
                    r#type: Type::Ident("Nat"),
                    body: Box::new(Term::Return(Box::new(Term::Add(
                        Box::new(Term::Var("n".to_string())),
                        Box::new(Term::Nat(1))
                    )))) }
            ])
        );
    }
}