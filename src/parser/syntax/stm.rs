use std::collections::{HashMap, HashSet};

use crate::cbpv::{term::Term, translate::Translate};

use super::{decl::Decl, expr::Expr, r#type::Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stm<'a> {
    If {
        cond: Box<Stm<'a>>,
        then: Box<Stm<'a>>,
        r#else: Box<Stm<'a>>
    },
    Let {
        var: &'a str,
        val: Box<Stm<'a>>,
        body: Box<Stm<'a>>
    },
    Exists {
        var: &'a str,
        r#type: Type<'a>,
        body: Box<Stm<'a>>
    },
    Equate {
        lhs: Expr<'a>,
        rhs: Expr<'a>,
        body: Box<Stm<'a>>
    },
    Choice(Vec<Expr<'a>>),
    Expr(Expr<'a>)
}

impl<'a> Translate<'a> for Stm<'a> {
    fn translate(self, vars: &mut HashSet<String>, funcs: &mut HashMap<String, Decl<'a>>) -> crate::cbpv::term::Term<'a> {
        match self {
            Stm::If { cond, then, r#else } => {
                let x = vars.len().to_string();
                vars.insert(x.clone());

                let cond = cond.translate(vars, funcs);
                let then = then.translate(vars, funcs);
                let r#else = r#else.translate(vars, funcs);

                vars.remove(&x);

                Term::Bind {
                    var: x.clone(),
                    val: Box::new(cond),
                    body: Box::new(Term::If {
                        cond: Box::new(Term::Var(x)),
                        then: Box::new(then),
                        r#else: Box::new(r#else)
                    })
                }
            },
            Stm::Let { var, val, body } => {
                let flag = vars.insert(var.to_string());

                let val = val.translate(vars, funcs);
                let body = body.translate(vars, funcs);

                if flag { vars.remove(var); }

                Term::Bind {
                    var: var.to_string(),
                    val: Box::new(val),
                    body: Box::new(body)
                }
            },
            Stm::Exists { var, r#type, body } => {
                let flag = vars.insert(var.to_string());

                let body = body.translate(vars, funcs);

                if flag { vars.remove(var); }

                Term::Exists {
                    var,
                    r#type: r#type.clone(),
                    body: Box::new(body)
                }
            },
            Stm::Equate { lhs, rhs, body } => {
                let x = vars.len().to_string();
                vars.insert(x.clone());
                let y = vars.len().to_string();
                vars.insert(y.clone());

                let lhs = lhs.translate(vars, funcs);
                let rhs = rhs.translate(vars, funcs);
                let body = body.translate(vars, funcs);

                vars.remove(&x);
                vars.remove(&y);

                Term::Bind {
                    var: x.clone(),
                    val: Box::new(lhs),
                    body: Box::new(Term::Bind {
                        var: y.clone(),
                        val: Box::new(rhs),
                        body: Box::new(Term::Equate {
                            lhs: Box::new(Term::Var(x)),
                            rhs: Box::new(Term::Var(y)),
                            body: Box::new(body)
                        })
                    })
                }
            },
            Stm::Choice(exprs) => Term::Choice(
                exprs.into_iter()
                    .map(|e| e.translate(vars, funcs)).collect()
            ),
            Stm::Expr(e) => e.translate(vars, funcs)
        }
    }
}