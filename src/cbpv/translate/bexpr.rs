use crate::{cbpv::Term, parser::syntax::bexpr::BExpr};

use super::Translate;

impl Translate for BExpr {
    fn translate(self) -> Term {
        match self {
            BExpr::Eq(lhs, rhs) => Term::Bind {
                var: "0".to_string(),
                val: Box::new(lhs.translate()),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(rhs.translate()),
                    body: Box::new(Term::Eq(
                        Box::new(Term::Var("0".to_string())),
                        Box::new(Term::Var("1".to_string()))
                    ))
                })
            },
            BExpr::NEq(lhs, rhs) => Term::Bind {
                var: "0".to_string(),
                val: Box::new(lhs.translate()),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(rhs.translate()),
                    body: Box::new(Term::NEq(
                        Box::new(Term::Var("0".to_string())),
                        Box::new(Term::Var("1".to_string()))
                    ))
                })
            },
            BExpr::Not(e) => Term::Bind {
                var: "".to_string(),
                val: Box::new(e.translate()),
                body: Box::new(Term::Not(Box::new(Term::Var("".to_string()))))
            }
        }
    }
}