use super::term::Term;

use crate::parser::syntax::{expr::Expr, program::Decl, stm::Stm};

pub fn translate<'a>(decl: &'a Decl<'a>) -> Term<'a> {
    match decl {
        Decl::Func { name: _, args, body } => Term::Return(
            Box::new(Term::Thunk(Box::new(Term::Lambda {
                args: args.clone(),
                body: Box::new(translate_stm(body))
            })))
        ),
        Decl::Stm(s) => translate_stm(s),
        _ => unreachable!()
    }
}

fn translate_stm<'a>(stm: &'a Stm<'a>) -> Term<'a> {
    match stm {
        Stm::If { cond, then, r#else } => Term::Bind {
            var: "x",
            val: Box::new(translate_stm(cond)),
            body: Box::new(Term::If {
                cond: Box::new(Term::Var("x")),
                then: Box::new(translate_stm(then)),
                r#else: Box::new(translate_stm(r#else))
            })
        },
        Stm::Let { var, val, body } => Term::Bind {
            var: *var,
            val: Box::new(translate_stm(val)),
            body: Box::new(translate_stm(body))
        },
        Stm::Exists { var, r#type, body } => Term::Exists {
            var: *var,
            r#type: r#type.clone(),
            body: Box::new(translate_stm(body))
        },
        Stm::Equate { lhs, rhs, body } => Term::Equate {
            lhs: Box::new(translate_expr(lhs)),
            rhs: Box::new(translate_expr(rhs)),
            body: Box::new(translate_stm(body))
        },
        Stm::Choice(exprs) => Term::Choice(
            exprs.iter().map(translate_expr).collect()
        ),
        Stm::Expr(e) => translate_expr(e)
    }
}

fn translate_expr<'a>(expr: &'a Expr<'a>) -> Term<'a> {
    match expr {
        Expr::App(lhs, rhs) => Term::Bind {
            var: "x",
            val: Box::new(translate_expr(rhs)),
            body: Box::new(Term::Bind {
                var: "f",
                val: Box::new(translate_expr(lhs)),
                body: Box::new(Term::App(
                    Box::new(Term::Force(Box::new(Term::Var("f")))),
                    Box::new(Term::Var("x"))
                ))
            })
        },
        Expr::Ident(s) => Term::Return(
            Box::new(Term::Var(s))
        ),
        Expr::Nat(n) => Term::Return(
            Box::new(Term::Nat(*n))
        ),
        Expr::Stm(s) => translate_stm(s)
    }
}