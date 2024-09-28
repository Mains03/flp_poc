use crate::{cbpv::{term_ptr::TermPtr, Term}, parser::syntax::{expr::Expr, stm::Stm}};

use super::Translate;

impl Translate for Expr {
    fn translate(self) -> Term {
        match self {
            Expr::Cons(x, xs) => Term::Bind {
                var: "0".to_string(),
                val: TermPtr::from_term(x.translate()),
                body: TermPtr::from_term(Term::Bind {
                    var: "1".to_string(),
                    val: TermPtr::from_term(xs.translate()),
                    body: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Cons(
                        TermPtr::from_term(Term::Var("0".to_string())),
                        TermPtr::from_term(Term::Var("1".to_string()))
                    ))))
                })
            },
            Expr::Add(lhs, rhs) => Term::Bind {
                var: "0".to_string(),
                val: TermPtr::from_term(lhs.translate()),
                body: TermPtr::from_term(Term::Bind {
                    var: "1".to_string(),
                    val: TermPtr::from_term(rhs.translate()),
                    body: TermPtr::from_term(Term::Add(
                        "0".to_string(),
                        "1".to_string()
                    ))
                })
            },
            Expr::App(lhs, rhs) => Term::Bind {
                var: "0".to_string(),
                val: TermPtr::from_term(rhs.translate()),
                body: TermPtr::from_term(Term::Bind {
                    var: "1".to_string(),
                    val: TermPtr::from_term(lhs.translate()),
                    body: TermPtr::from_term(Term::App(
                        TermPtr::from_term(Term::Force("1".to_string())),
                        "0".to_string()
                    ))
                })
            },
            Expr::BExpr(bexpr) => bexpr.translate(),
            Expr::List(mut elems) => {
                elems.reverse();
                translate_list(elems, 0, vec![])
            },
            Expr::Lambda(arg, body) => {
                let body = body.translate();

                Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(
                    Term::Lambda {
                        arg, body: TermPtr::from_term(body)
                    }
                ))))
            },
            Expr::Ident(s) => Term::Return(TermPtr::from_term(Term::Var(s.clone()))),
            Expr::Nat(n) => Term::Return(TermPtr::from_term(translate_nat(n))),
            Expr::Bool(b) => Term::Return(TermPtr::from_term(Term::Bool(b))),
            Expr::Pair(lhs, rhs) => translate_pair(*lhs, *rhs),
            Expr::Stm(s) => s.translate()
        }
    }
}

fn translate_list(mut elems: Vec<Expr>, i: usize, mut list: Vec<Term>) -> Term {
    if elems.len() == 0 {
        list.reverse();

        Term::Return(TermPtr::from_term(
            list.into_iter()
                .fold(Term::Nil, |acc, t| {
                    Term::Cons(TermPtr::from_term(t), TermPtr::from_term(acc))
                })
        ))
    } else {
        let item = elems.remove(elems.len()-1).translate();
        list.push(Term::Var(i.to_string()));

        Term::Bind {
            var: i.to_string(),
            val: TermPtr::from_term(item),
            body: TermPtr::from_term(
                translate_list(elems, i+1, list)
            )
        }
    }
}

fn translate_nat(n: usize) -> Term {
    if n == 0 {
        Term::Zero
    } else {
        Term::Succ(TermPtr::from_term(translate_nat(n-1)))
    }
}

fn translate_pair(lhs: Stm, rhs: Stm) -> Term {
    Term::Bind {
        var: "x".to_string(),
        val: TermPtr::from_term(lhs.translate()),
        body: TermPtr::from_term(Term::Bind {
            var: "y".to_string(),
            val: TermPtr::from_term(rhs.translate()),
            body: TermPtr::from_term(Term::Return(TermPtr::from_term(
                Term::Pair(
                    TermPtr::from_term(Term::Var("x".to_string())),
                    TermPtr::from_term(Term::Var("y".to_string()))
                )
            )))
        })
    }
}