use std::collections::{HashMap, HashSet};

use crate::cbpv::{term::Term, translate::Translate};

use super::{decl::Decl, expr::Expr};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BExpr<'a> {
    Eq(Box<Expr<'a>>, Box<Expr<'a>>),
    NEq(Box<Expr<'a>>, Box<Expr<'a>>),
    Not(Box<Expr<'a>>)
}

impl<'a> Translate<'a> for BExpr<'a> {
    fn translate(self, vars: &mut HashSet<String>, funcs: &mut HashMap<String, Decl<'a>>) -> crate::cbpv::term::Term<'a> {
        match self {
            BExpr::Eq(lhs, rhs) => {
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
                        body: Box::new(Term::Eq(Box::new(Term::Var(x)), Box::new(Term::Var(y))))
                    })
                }
            },
            BExpr::NEq(lhs, rhs) => {
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
                        body: Box::new(Term::NEq(Box::new(Term::Var(x)), Box::new(Term::Var(y))))
                    })
                }
            },
            BExpr::Not(e) => {
                let x = vars.len().to_string();
                vars.insert(x.clone());
    
                let e = e.translate(vars, funcs);
    
                vars.remove(&x);
    
                Term::Bind {
                    var: x.clone(),
                    val: Box::new(e),
                    body: Box::new(Term::Not(Box::new(Term::Var(x))))
                }
            }
        }
    }
}