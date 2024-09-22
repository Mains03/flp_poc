use crate::{cbpv::{pm::{PMList, PMListCons, PMNat, PMNatSucc, PM}, term_ptr::TermPtr, Term}, parser::syntax::{case::Case, stm::Stm}};

use super::Translate;

impl Translate for Stm {
    fn translate(self) -> Term {
        match self {
            Stm::If { cond, then, r#else } => Term::Bind {
                var: "".to_string(),
                val: TermPtr::from_term(cond.translate()),
                body: TermPtr::from_term(Term::If {
                    cond: "".to_string(),
                    then: TermPtr::from_term(then.translate()),
                    r#else: TermPtr::from_term(r#else.translate())
                })
            },
            Stm::Let { var, val, body } => Term::Bind {
                var: var,
                val: TermPtr::from_term(val.translate()),
                body: TermPtr::from_term(body.translate())
            },
            Stm::Exists { var, r#type: _, body } => Term::Exists {
                var,
                body: TermPtr::from_term(body.translate())
            },
            Stm::Equate { lhs, rhs, body } => Term::Bind {
                var: "0".to_string(),
                val: TermPtr::from_term(lhs.translate()),
                body: TermPtr::from_term(Term::Bind {
                    var: "1".to_string(),
                    val: TermPtr::from_term(rhs.translate()),
                    body: TermPtr::from_term(Term::Equate {
                        lhs: "0".to_string(),
                        rhs: "1".to_string(),
                        body: TermPtr::from_term(body.translate())
                    })
                })
            },
            Stm::Choice(exprs) => Term::Choice(
                exprs.into_iter()
                    .map(|e| TermPtr::from_term(e.translate())).collect()
            ),
            Stm::Case(var, case) => Term::PM(match case {
                Case::Nat(nat_case) => {
                    let succ = nat_case.succ.unwrap();

                    PM::PMNat(PMNat {
                        var,
                        zero: TermPtr::from_term(nat_case.zero.unwrap().expr.translate()),
                        succ: PMNatSucc {
                            var: succ.var,
                            body: TermPtr::from_term(succ.expr.translate())
                        }
                    })
                },
                Case::List(list_case) => {
                    let cons = list_case.cons.unwrap();

                    PM::PMList(PMList {
                        var,
                        nil: TermPtr::from_term(list_case.empty.unwrap().expr.translate()),
                        cons: PMListCons {
                            x: cons.x,
                            xs: cons.xs,
                            body: TermPtr::from_term(cons.expr.translate())
                        }
                    })
                }
            }),
            Stm::Expr(e) => e.translate()
        }
    }
}