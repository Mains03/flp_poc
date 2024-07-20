use std::collections::{HashMap, HashSet};

use crate::cbpv::{term::Term, translate::Translate};

use super::{bexpr::BExpr, decl::Decl, stm::Stm};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr<'a> {
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    App(Box<Expr<'a>>, Box<Expr<'a>>),
    BExpr(BExpr<'a>),
    Ident(&'a str),
    Nat(usize),
    Bool(bool),
    Stm(Box<Stm<'a>>)
}

impl<'a> Translate<'a> for Expr<'a> {
    fn translate(self, vars: &mut HashSet<String>, funcs: &mut HashMap<String, Decl<'a>>) -> crate::cbpv::term::Term<'a> {
        match self {
            Expr::Add(lhs, rhs) => {
                let x = vars.len().to_string();
                vars.insert(x.clone());
                let y = vars.len().to_string();
                vars.insert(y.clone());
    
                let lhs = lhs.translate(vars, funcs);
                let rhs = rhs.translate(vars, funcs);
    
                vars.remove(&x);
                vars.remove(&y);
    
                Term::Bind {
                    var: x.clone(),
                    val: Box::new(lhs),
                    body: Box::new(Term::Bind {
                        var: y.clone(),
                        val: Box::new(rhs),
                        body: Box::new(Term::Add(Box::new(Term::Var(x)), Box::new(Term::Var(y))))
                    })
                }
            },
            Expr::App(lhs, rhs) => {
                let x = vars.len().to_string();
                vars.insert(x.clone());
                let f = vars.len().to_string();
                vars.insert(f.clone());
    
                let lhs = lhs.translate(vars, funcs);
                let rhs = rhs.translate(vars, funcs);
    
                vars.remove(&x);
                vars.remove(&f);
    
                Term::Bind {
                    var: x.clone(),
                    val: Box::new(rhs),
                    body: Box::new(Term::Bind {
                        var: f.clone(),
                        val: Box::new(lhs),
                        body: Box::new(Term::App(
                            Box::new(Term::Force(Box::new(Term::Var(f)))),
                            Box::new(Term::Var(x))
                        ))
                    })
                }
            },
            Expr::BExpr(bexpr) => bexpr.translate(vars, funcs),
            Expr::Ident(s) => {
                if funcs.contains_key(s) {
                    funcs.get(s).unwrap().clone().translate(vars, funcs)
                } else {
                    Term::Return(Box::new(Term::Var(s.to_string())))
                }
            },
            Expr::Nat(n) => Term::Return(Box::new(translate_nat(n))),
            Expr::Bool(b) => Term::Return(Box::new(Term::Bool(b))),
            Expr::Stm(s) => s.translate(vars, funcs)
        }
    }
}

fn translate_nat<'a>(n: usize) -> Term<'a> {
    if n == 0 {
        Term::Zero
    } else {
        Term::Succ(Box::new(translate_nat(n-1)))
    }
}