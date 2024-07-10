use std::collections::HashSet;

use super::term::Term;

use crate::parser::syntax::{expr::Expr, program::Decl, stm::Stm};

pub fn translate<'a>(decl: Decl<'a>) -> Term<'a> {
    let mut vars: HashSet<String> = HashSet::new();
    
    match decl {
        Decl::Func { name: _, mut args, body } => if args.len() == 0 {
            translate_stm(body, &mut vars)
        } else {
            args.iter()
                .for_each(|s| { vars.insert(s.to_string()); } );

            // reverse so that application uses variable at the end of the list
            args.reverse();

            Term::Return(Box::new(
                Term::Thunk(Box::new(
                    Term::Lambda {
                        args,
                        body: Box::new(translate_stm(body, &mut vars))
                    }
                ))
            ))
        },
        Decl::Stm(s) => translate_stm(s, &mut vars),
        _ => unreachable!()
    }
}

fn translate_stm<'a>(stm: Stm<'a>, vars: &mut HashSet<String>) -> Term<'a> {
    match stm {
        Stm::If { cond, then, r#else } => {
            let x = vars.len().to_string();
            vars.insert(x.clone());

            let t = Term::Bind {
                var: x.clone(),
                val: Box::new(translate_stm(*cond, vars)),
                body: Box::new(Term::If {
                    cond: Box::new(Term::Var(x.clone())),
                    then: Box::new(translate_stm(*then, vars)),
                    r#else: Box::new(translate_stm(*r#else, vars))
                })
            };

            vars.remove(&x);

            t
        },
        Stm::Let { var, val, body } => {
            let flag = vars.insert(var.to_string());

            let t = Term::Bind {
                var: var.to_string(),
                val: Box::new(translate_stm(*val, vars)),
                body: Box::new(translate_stm(*body, vars))
            };

            if flag { vars.remove(var); }

            t
        },
        Stm::Exists { var, r#type, body } => {
            let flag = vars.insert(var.to_string());

            let t = Term::Exists {
                var,
                r#type: r#type.clone(),
                body: Box::new(translate_stm(*body, vars))
            };

            if flag { vars.remove(var); }

            t
        },
        Stm::Equate { lhs, rhs, body } => {
            let var = vars.len().to_string();
            vars.insert(var.clone());

            Term::Equate {
                lhs: Box::new(Term::Bind {
                    var: var.clone(),
                    val: Box::new(translate_expr(lhs, vars)),
                    body: Box::new(Term::Var(var.clone()))
                }),
                rhs: Box::new(Term::Bind {
                    var: var.clone(),
                    val: Box::new(translate_expr(rhs, vars)),
                    body: Box::new(Term::Var(var))
                }),
                body: Box::new(translate_stm(*body, vars))
            }
        },
        Stm::Choice(exprs) => Term::Choice(
            exprs.into_iter()
                .map(|e| translate_expr(e,vars)).collect()
        ),
        Stm::Expr(e) => translate_expr(e, vars)
    }
}

fn translate_expr<'a>(expr: Expr<'a>, vars: &mut HashSet<String>) -> Term<'a> {
    match expr {
        Expr::Add(lhs, rhs) => translate_expr(
            Expr::App(Box::new(
                Expr::App(Box::new(Expr::Ident("+")),
                lhs
            )), rhs), vars),
        Expr::App(lhs, rhs) => {
            let x = vars.len().to_string();
            vars.insert(x.clone());
            let f = vars.len().to_string();
            vars.insert(f.clone());

            let t = Term::Bind {
                var: x.clone(),
                val: Box::new(translate_expr(*rhs, vars)),
                body: Box::new(Term::Bind {
                    var: f.clone(),
                    val: Box::new(translate_expr(*lhs, vars)),
                    body: Box::new(Term::App(
                        Box::new(Term::Force(Box::new(Term::Var(f.clone())))),
                        Box::new(Term::Var(x.clone()))
                    ))
                })
            };

            vars.remove(&x);
            vars.remove(&f);

            t
        },
        Expr::Ident(s) => {
            if vars.contains(s) {
                Term::Return(
                    Box::new(Term::Var(s.to_string()))
                )
            } else {
                Term::Var(s.to_string())
            }
        },
        Expr::Nat(n) => Term::Return(
            Box::new(Term::Nat(n))
        ),
        Expr::Stm(s) => translate_stm(*s, vars)
    }
}

#[cfg(test)]
mod test {
    use crate::parser::{self, syntax::r#type::Type};

    use super::*;

    #[test]
    fn test1() {
        let src = "const :: a -> b -> a
const x y = x.";

        let mut ast = parser::parse(src).unwrap();

        let cbpv = translate(ast.remove(1));

        assert_eq!(
            cbpv,
            Term::Return(Box::new(
                Term::Thunk(Box::new(
                    Term::Lambda {
                        args: vec!["y", "x"],
                        body: Box::new(Term::Return(
                            Box::new(Term::Var("x".to_string()))
                        ))
                    }
                ))
            ))
        );
    }

    #[test]
    fn test2() {
        let src = "id :: Nat -> Nat
id x = exists n :: Nat. n =:= x. n.";

        let mut ast = parser::parse(src).unwrap();

        let cbpv = translate(ast.remove(1));

        assert_eq!(
            cbpv,
            Term::Return(Box::new(
                Term::Thunk(Box::new(
                    Term::Lambda {
                        args: vec!["x"],
                        body: Box::new(Term::Exists {
                            var: "n",
                            r#type: Type::Ident("Nat"),
                            body: Box::new(Term::Equate {
                                lhs: Box::new(Term::Bind {
                                    var: "2".to_string(),
                                    val: Box::new(Term::Return(
                                        Box::new(Term::Var("n".to_string()))
                                    )),
                                    body: Box::new(Term::Var("2".to_string()))
                                }),
                                rhs: Box::new(Term::Bind {
                                    var: "2".to_string(),
                                    val: Box::new(Term::Return(
                                        Box::new(Term::Var("x".to_string()))
                                    )),
                                    body: Box::new(Term::Var("2".to_string()))
                                }),
                                body: Box::new(Term::Return(
                                    Box::new(Term::Var("n".to_string()))
                                ))
                            })
                        })
                    }
                ))
            ))
        );
    }

    #[test]
    fn test3() {
        let src = "id :: Nat -> Nat
id x = exists n :: Nat. n =:= x. n.

let x = 5 in id x.";

        let mut ast = parser::parse(src).unwrap();

        let cbpv = translate(ast.remove(2));

        assert_eq!(
            cbpv,
            Term::Bind {
                var: "x".to_string(),
                val: Box::new(Term::Return(
                    Box::new(Term::Nat(5))
                )),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Return(
                        Box::new(Term::Var("x".to_string()))
                    )),
                    body: Box::new(Term::Bind {
                        var: "2".to_string(),
                        val: Box::new(Term::Var("id".to_string())),
                        body: Box::new(Term::App(
                            Box::new(Term::Force(
                                Box::new(Term::Var("2".to_string()))
                            )),
                            Box::new(Term::Var("1".to_string())
                        )))
                    })
                })
            }
        )
    }

    #[test]
    fn test4() {
        let src = "id :: a -> a
id x = x.

1 <> id 2 <> 3.";

        let mut ast = parser::parse(src).unwrap();
        
        let cbpv = translate(ast.remove(2));

        assert_eq!(
            cbpv,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Nat(1))),
                Term::Bind {
                    var: "0".to_string(),
                    val: Box::new(Term::Return(Box::new(Term::Nat(2)))),
                    body: Box::new(Term::Bind {
                        var: "1".to_string(),
                        val: Box::new(Term::Var("id".to_string())),
                        body: Box::new(Term::App(
                            Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                            Box::new(Term::Var("0".to_string()))
                        ))
                    })
                },
                Term::Return(Box::new(Term::Nat(3)))
            ])
        )
    }

    #[test]
    fn test5() {
        let src = "addOne :: Nat -> Nat
addOne n = n + 1.";

        let mut ast = parser::parse(src).unwrap();
        let cbpv = translate(ast.remove(1));

        assert_eq!(
            cbpv,
            Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                args: vec!["n"],
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Return(Box::new(Term::Nat(1)))),
                    body: Box::new(Term::Bind {
                        var: "2".to_string(),
                        val: Box::new(Term::Bind {
                            var: "3".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Var("n".to_string())))),
                            body:  Box::new(Term::Bind {
                                var: "4".to_string(),
                                val: Box::new(Term::Var("+".to_string())),
                                body: Box::new(Term::App(
                                    Box::new(Term::Force(Box::new(Term::Var("4".to_string())))),
                                    Box::new(Term::Var("3".to_string()))
                                ))
                            })
                        }),
                        body: Box::new(Term::App(
                            Box::new(Term::Force(Box::new(Term::Var("2".to_string())))),
                            Box::new(Term::Var("1".to_string()))
                        ))
                    })
                })
            }))))
        )
    }
}