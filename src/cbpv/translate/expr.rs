use crate::{cbpv::Term, parser::syntax::expr::Expr};

use super::Translate;

impl Translate for Expr {
    fn translate(self) -> Term {
        match self {
            Expr::Add(lhs, rhs) => Term::Bind {
                var: "0".to_string(),
                val: Box::new(lhs.translate()),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(rhs.translate()),
                    body: Box::new(Term::Add(
                        "0".to_string(),
                        "1".to_string()
                    ))
                })
            },
            Expr::App(lhs, rhs) => Term::Bind {
                var: "0".to_string(),
                val: Box::new(rhs.translate()),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(lhs.translate()),
                    body: Box::new(Term::App(
                        Box::new(Term::Force("1".to_string())),
                        "0".to_string()
                    ))
                })
            },
            Expr::BExpr(bexpr) => bexpr.translate(),
            Expr::List(mut elems) => {
                elems.reverse();
                translate_list(elems, 0, vec![])
            },
            Expr::Lambda(var, body) => {
                let body = body.translate();
                
                let mut free_vars = body.free_vars();
                free_vars.remove(&var);

                Term::Return(Box::new(Term::Thunk(Box::new(
                    Term::Lambda {
                        var,
                        free_vars,
                        body: Box::new(body)
                    }
                ))))
            },
            Expr::Ident(s) => Term::Return(Box::new(Term::Var(s.clone()))),
            Expr::Nat(n) => Term::Return(Box::new(translate_nat(n))),
            Expr::Bool(b) => Term::Return(Box::new(Term::Bool(b))),
            Expr::Fold => Term::Fold,
            Expr::Stm(s) => s.translate()
        }
    }
}

fn translate_list(mut elems: Vec<Expr>, i: usize, mut list: Vec<Term>) -> Term {
    if elems.len() == 0 {
        list.reverse();

        Term::Return(Box::new(
            list.into_iter()
                .fold(Term::Nil, |acc, t| {
                    Term::Cons(Box::new(t), Box::new(acc))
                })
        ))
    } else {
        let item = elems.remove(elems.len()-1).translate();
        list.push(Term::Var(i.to_string()));

        Term::Bind {
            var: i.to_string(),
            val: Box::new(item),
            body: Box::new(
                translate_list(elems, i+1, list)
            )
        }
    }
}

fn translate_nat(n: usize) -> Term {
    if n == 0 {
        Term::Zero
    } else {
        Term::Succ(Box::new(translate_nat(n-1)))
    }
}