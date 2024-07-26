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
                        Box::new(Term::Var("0".to_string())),
                        Box::new(Term::Var("1".to_string()))
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
                        Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                        Box::new(Term::Var("0".to_string()))
                    ))
                })
            },
            Expr::BExpr(bexpr) => bexpr.translate(),
            Expr::Ident(s) => Term::Return(Box::new(Term::Var(s.clone()))),
            Expr::Nat(n) => Term::Return(Box::new(translate_nat(n))),
            Expr::Bool(b) => Term::Return(Box::new(Term::Bool(b))),
            Expr::Stm(s) => s.translate()
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