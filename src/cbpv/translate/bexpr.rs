use crate::{cbpv::{term_ptr::TermPtr, Term}, parser::syntax::bexpr::BExpr};

use super::Translate;

impl Translate for BExpr {
    fn translate(self) -> Term {
        match self {
            BExpr::Eq(lhs, rhs) => Term::Bind {
                var: "0".to_string(),
                val: TermPtr::from_term(lhs.translate()),
                body: TermPtr::from_term(Term::Bind {
                    var: "1".to_string(),
                    val: TermPtr::from_term(rhs.translate()),
                    body: TermPtr::from_term(Term::Eq(
                        "0".to_string(),
                        "1".to_string()
                    ))
                })
            },
            BExpr::NEq(lhs, rhs) => Term::Bind {
                var: "0".to_string(),
                val: TermPtr::from_term(lhs.translate()),
                body: TermPtr::from_term(Term::Bind {
                    var: "1".to_string(),
                    val: TermPtr::from_term(rhs.translate()),
                    body: TermPtr::from_term(Term::NEq(
                        "0".to_string(),
                        "1".to_string()
                    ))
                })
            },
            BExpr::Not(e) => Term::Bind {
                var: "".to_string(),
                val: TermPtr::from_term(e.translate()),
                body: TermPtr::from_term(Term::Not("".to_string()))
            }
        }
    }
}