use std::cell::RefCell;

use super::term::Term;

use crate::parser::syntax::{expr::Expr, program::Decl, stm::Stm};

pub fn translate<'a>(decl: &'a Decl<'a>) -> Term<'a> {
    let i = RefCell::new(0);

    match decl {
        Decl::Func { name: _, args, body } => Term::Thunk(Box::new(Term::Lambda {
            args: args.clone(),
            body: Box::new(translate_stm(body, &i))
        })),
        Decl::Stm(s) => translate_stm(s, &i),
        _ => unreachable!()
    }
}

fn translate_stm<'a>(stm: &'a Stm<'a>, i: &RefCell<usize>) -> Term<'a> {
    match stm {
        Stm::If { cond, then, r#else } => {
            let val = i.replace_with(|n| *n+1);
            let x = val.to_string();

            Term::Bind {
                var: x.clone(),
                val: Box::new(translate_stm(cond, i)),
                body: Box::new(Term::If {
                    cond: Box::new(Term::Var(x)),
                    then: Box::new(translate_stm(then, i)),
                    r#else: Box::new(translate_stm(r#else, i))
                })
            }
        },
        Stm::Let { var, val, body } => Term::Bind {
            var: var.to_string(),
            val: Box::new(translate_stm(val, i)),
            body: Box::new(translate_stm(body, i))
        },
        Stm::Exists { var, r#type, body } => Term::Exists {
            var: *var,
            r#type: r#type.clone(),
            body: Box::new(translate_stm(body, i))
        },
        Stm::Equate { lhs, rhs, body } => Term::Equate {
            lhs: Box::new(translate_expr(lhs, i)),
            rhs: Box::new(translate_expr(rhs, i)),
            body: Box::new(translate_stm(body, i))
        },
        Stm::Choice(exprs) => Term::Choice(
            exprs.iter().map(|e| translate_expr(e,i)).collect()
        ),
        Stm::Expr(e) => translate_expr(e, i)
    }
}

fn translate_expr<'a>(expr: &'a Expr<'a>, i: &RefCell<usize>) -> Term<'a> {
    match expr {
        Expr::App(lhs, rhs) => {
            let val = i.replace_with(|n| *n+2);
            let x: String = val.to_string();
            let f = (val+1).to_string();

            Term::Bind {
                var: x.clone(),
                val: Box::new(translate_expr(rhs, i)),
                body: Box::new(Term::Bind {
                    var: f.clone(),
                    val: Box::new(translate_expr(lhs, i)),
                    body: Box::new(Term::App(
                        Box::new(Term::Force(Box::new(Term::Var(f)))),
                        Box::new(Term::Var(x))
                    ))
                })
            }
        },
        Expr::Ident(s) => Term::Return(
            Box::new(Term::Var(s.to_string()))
        ),
        Expr::Nat(n) => Term::Return(
            Box::new(Term::Nat(*n))
        ),
        Expr::Stm(s) => translate_stm(s, i)
    }
}

#[cfg(test)]
mod test {
    use crate::parser::{self, syntax::r#type::Type};

    use super::*;

    #[test]
    fn test1() {
        let src = "id :: Nat -> Nat
id x = exists n :: Nat. n =:= x. n";

        let ast = parser::parse(src).unwrap();

        let cbpv = translate(ast.get(1).unwrap());

        assert_eq!(
            cbpv,
            Term::Thunk(Box::new(
                Term::Lambda {
                    args: vec!["x"],
                    body: Box::new(Term::Exists {
                        var: "n",
                        r#type: Type::Ident("Nat"),
                        body: Box::new(Term::Equate {
                            lhs: Box::new(Term::Return(
                                Box::new(Term::Var("n".to_string()))
                            )),
                            rhs: Box::new(Term::Return(
                                Box::new(Term::Var("x".to_string()))
                            )),
                            body: Box::new(Term::Return(
                                Box::new(Term::Var("n".to_string()))
                            ))
                        })
                    })
                }
            ))
        );
    }

    #[test]
    fn test2() {
        let src = "id :: Nat -> Nat
id x = exists n :: Nat. n =:= x. n

let x = 5 in id x";

        let ast = parser::parse(src).unwrap();

        let cbpv = translate(ast.get(2).unwrap());

        assert_eq!(
            cbpv,
            Term::Bind {
                var: "x".to_string(),
                val: Box::new(Term::Return(
                    Box::new(Term::Nat(5))
                )),
                body: Box::new(Term::Bind {
                    var: "0".to_string(),
                    val: Box::new(Term::Return(
                        Box::new(Term::Var("x".to_string()))
                    )),
                    body: Box::new(Term::Bind {
                        var: "1".to_string(),
                        val: Box::new(Term::Return(
                            Box::new(Term::Var("id".to_string()))
                        )),
                        body: Box::new(Term::App(
                            Box::new(Term::Force(
                                Box::new(Term::Var("1".to_string()))
                            )),
                            Box::new(Term::Var("0".to_string())
                        )))
                    })
                })
            }
        )
    }
}