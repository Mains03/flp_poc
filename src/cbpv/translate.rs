use std::collections::HashMap;

use crate::parser::syntax::{arg::Arg, bexpr::BExpr, case::Case, decl::Decl, expr::Expr, stm::Stm};

use super::{pm::{PMList, PMListCons, PMNat, PMNatSucc, PM}, term_ptr::TermPtr, Term};

pub fn translate(ast: Vec<Decl>) -> HashMap<String, Term> {
    let mut cbpv = HashMap::new();

    ast.into_iter()
        .for_each(|decl| match decl {
            Decl::FuncType { name: _, r#type: _ } => (),
            Decl::Func { name, args, body } => {
                cbpv.insert(
                    name,
                    translate_func(args, body)
                );
            },
            Decl::Stm(stm) => {
                cbpv.insert("main".to_string(), translate_stm(stm));
            }
        });

    cbpv
}

fn translate_func(mut args: Vec<Arg>, body: Stm) -> Term {
    args.reverse();

    if args.len() > 0 {
        let arg = args.remove(args.len()-1);
        let body = translate_func_helper(args, body);

        Term::Thunk(TermPtr::from_term(Term::Lambda {
            arg, body: TermPtr::from_term(body)
        }))
    } else {
        translate_func_helper(args, body)
    }
}

fn translate_func_helper(mut args: Vec<Arg>, body: Stm) -> Term {
    if args.len() == 0 {
        translate_stm(body)
    } else {
        let arg = args.remove(args.len()-1);
        let body = translate_func_helper(args, body);

        Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(Term::Lambda {
            arg, body: TermPtr::from_term(body)
        }))))
    }
}


fn translate_stm(stm: Stm) -> Term {
    match stm {
        Stm::If { cond, then, r#else } => Term::Bind {
            var: "".to_string(),
            val: TermPtr::from_term(translate_stm(*cond)),
            body: TermPtr::from_term(Term::If {
                cond: "".to_string(),
                then: TermPtr::from_term(translate_stm(*then)),
                r#else: TermPtr::from_term(translate_stm(*r#else))
            })
        },
        Stm::Let { var, val, body } => Term::Bind {
            var: var,
            val: TermPtr::from_term(translate_stm(*val)),
            body: TermPtr::from_term(translate_stm(*body))
        },
        Stm::Exists { var, r#type: _, body } => Term::Exists {
            var,
            body: TermPtr::from_term(translate_stm(*body))
        },
        Stm::Equate { lhs, rhs, body } => Term::Bind {
            var: "0".to_string(),
            val: TermPtr::from_term(translate_expr(lhs)),
            body: TermPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: TermPtr::from_term(translate_expr(rhs)),
                body: TermPtr::from_term(Term::Equate {
                    lhs: "0".to_string(),
                    rhs: "1".to_string(),
                    body: TermPtr::from_term(translate_stm(*body))
                })
            })
        },
        Stm::Choice(exprs) => Term::Choice(
            exprs.into_iter()
                .map(|e| TermPtr::from_term(translate_expr(e))).collect()
        ),
        Stm::Case(var, case) => Term::PM(match case {
            Case::Nat(nat_case) => {
                let succ = nat_case.succ.unwrap();

                PM::PMNat(PMNat {
                    var,
                    zero: TermPtr::from_term(translate_expr(nat_case.zero.unwrap().expr)),
                    succ: PMNatSucc {
                        var: succ.var,
                        body: TermPtr::from_term(translate_expr(succ.expr))
                    }
                })
            },
            Case::List(list_case) => {
                let cons = list_case.cons.unwrap();

                PM::PMList(PMList {
                    var,
                    nil: TermPtr::from_term(translate_expr(list_case.empty.unwrap().expr)),
                    cons: PMListCons {
                        x: cons.x,
                        xs: cons.xs,
                        body: TermPtr::from_term(translate_expr(cons.expr))
                    }
                })
            }
        }),
        Stm::Expr(e) => translate_expr(e)
    }
}

fn translate_expr(expr: Expr) -> Term {
    match expr {
        Expr::Cons(x, xs) => Term::Bind {
            var: "0".to_string(),
            val: TermPtr::from_term(translate_expr(*x)),
            body: TermPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: TermPtr::from_term(translate_expr(*xs)),
                body: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Cons(
                    TermPtr::from_term(Term::Var("0".to_string())),
                    TermPtr::from_term(Term::Var("1".to_string()))
                ))))
            })
        },
        Expr::Add(lhs, rhs) => Term::Bind {
            var: "0".to_string(),
            val: TermPtr::from_term(translate_expr(*lhs)),
            body: TermPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: TermPtr::from_term(translate_expr(*rhs)),
                body: TermPtr::from_term(Term::Add(
                    "0".to_string(),
                    "1".to_string()
                ))
            })
        },
        Expr::App(lhs, rhs) => Term::Bind {
            var: "0".to_string(),
            val: TermPtr::from_term(translate_expr(*rhs)),
            body: TermPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: TermPtr::from_term(translate_expr(*lhs)),
                body: TermPtr::from_term(Term::App(
                    TermPtr::from_term(Term::Force("1".to_string())),
                    "0".to_string()
                ))
            })
        },
        Expr::BExpr(bexpr) => translate_bexpr(bexpr),
        Expr::List(mut elems) => {
            elems.reverse();
            translate_list(elems, 0, vec![])
        },
        Expr::Lambda(arg, body) => {
            let body = translate_stm(*body);

            Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(
                Term::Lambda {
                    arg, body: TermPtr::from_term(body)
                }
            ))))
        },
        Expr::Ident(s) => Term::Return(TermPtr::from_term(Term::Var(s.clone()))),
        Expr::Nat(n) => Term::Return(TermPtr::from_term(translate_nat(n))),
        Expr::Bool(b) => Term::Return(TermPtr::from_term(Term::Bool(b))),
        Expr::Pair(lhs, rhs) => translate_pair(*lhs, *rhs),
        Expr::Stm(s) => translate_stm(*s)
    }
}

fn translate_bexpr(bexpr: BExpr) -> Term {
    match bexpr {
        BExpr::Eq(lhs, rhs) => Term::Bind {
            var: "0".to_string(),
            val: TermPtr::from_term(translate_expr(*lhs)),
            body: TermPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: TermPtr::from_term(translate_expr(*rhs)),
                body: TermPtr::from_term(Term::Eq(
                    "0".to_string(),
                    "1".to_string()
                ))
            })
        },
        BExpr::NEq(lhs, rhs) => Term::Bind {
            var: "0".to_string(),
            val: TermPtr::from_term(translate_expr(*lhs)),
            body: TermPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: TermPtr::from_term(translate_expr(*rhs)),
                body: TermPtr::from_term(Term::NEq(
                    "0".to_string(),
                    "1".to_string()
                ))
            })
        },
        BExpr::And(lhs, rhs) => Term::Bind {
            var: "0".to_string(),
            val: TermPtr::from_term(translate_expr(*lhs)),
            body: TermPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: TermPtr::from_term(translate_expr(*rhs)),
                body: TermPtr::from_term(Term::And(
                    TermPtr::from_term(Term::Var("0".to_string())),
                    TermPtr::from_term(Term::Var("1".to_string()))
                ))
            })
        },
        BExpr::Or(lhs, rhs) => Term::Bind {
            var: "0".to_string(),
            val: TermPtr::from_term(translate_expr(*lhs)),
            body: TermPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: TermPtr::from_term(translate_expr(*rhs)),
                body: TermPtr::from_term(Term::Or(
                    TermPtr::from_term(Term::Var("0".to_string())),
                    TermPtr::from_term(Term::Var("1".to_string()))
                ))
            })
        },
        BExpr::Not(e) => Term::Bind {
            var: "".to_string(),
            val: TermPtr::from_term(translate_expr(*e)),
            body: TermPtr::from_term(Term::Not("".to_string()))
        }
    }
}

fn translate_list(mut elems: Vec<Expr>, i: usize, mut list: Vec<Term>) -> Term {
    if elems.len() == 0 {
        list.reverse();

        Term::Return(TermPtr::from_term(
            list.into_iter()
                .fold(Term::Nil, |acc, t| {
                    Term::Cons(TermPtr::from_term(t), TermPtr::from_term(acc))
                })
        ))
    } else {
        let item = translate_expr(elems.remove(elems.len()-1));
        list.push(Term::Var(i.to_string()));

        Term::Bind {
            var: i.to_string(),
            val: TermPtr::from_term(item),
            body: TermPtr::from_term(
                translate_list(elems, i+1, list)
            )
        }
    }
}

fn translate_nat(n: usize) -> Term {
    if n == 0 {
        Term::Zero
    } else {
        Term::Succ(TermPtr::from_term(translate_nat(n-1)))
    }
}

fn translate_pair(lhs: Stm, rhs: Stm) -> Term {
    Term::Bind {
        var: "x".to_string(),
        val: TermPtr::from_term(translate_stm(lhs)),
        body: TermPtr::from_term(Term::Bind {
            var: "y".to_string(),
            val: TermPtr::from_term(translate_stm(rhs)),
            body: TermPtr::from_term(Term::Return(TermPtr::from_term(
                Term::Pair(
                    TermPtr::from_term(Term::Var("x".to_string())),
                    TermPtr::from_term(Term::Var("y".to_string()))
                )
            )))
        })
    }
}

#[cfg(test)]
mod test {
    use crate::parser::{self, syntax::arg::Arg};

    use super::*;

    #[test]
    fn test1() {
        let src = "const 1 2.";

        let mut cbpv = translate(parser::parse(src).unwrap());
        let term = cbpv.remove("main").unwrap();

        assert_eq!(
            term,
            Term::Bind {
                var: "0".to_string(),
                val: TermPtr::from_term(Term::Return(TermPtr::from_term(
                    Term::Succ(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero))))
                ))),
                body: TermPtr::from_term(Term::Bind {
                    var: "1".to_string(),
                    val: TermPtr::from_term(Term::Bind {
                        var: "0".to_string(),
                        val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero))))),
                        body: TermPtr::from_term(Term::Bind {
                            var: "1".to_string(),
                            val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("const".to_string())))),
                            body: TermPtr::from_term(Term::App(
                                TermPtr::from_term(Term::Force("1".to_string())),
                                "0".to_string()
                            ))
                        })
                    }),
                    body: TermPtr::from_term(Term::App(
                        TermPtr::from_term(Term::Force("1".to_string())),
                        "0".to_string()
                    ))
                })
            }
        );
    }

    #[test]
    fn test2() {
        let src = "let x = 1 in id x.";

        let mut cbpv = translate(parser::parse(src).unwrap());
        let term = cbpv.remove("main").unwrap();

        assert_eq!(
            term,
            Term::Bind {
                var: "x".to_string(),
                val: TermPtr::from_term(Term::Return(
                    TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero)))
                )),
                body: TermPtr::from_term(Term::Bind {
                    var: "0".to_string(),
                    val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("x".to_string())))),
                    body: TermPtr::from_term(Term::Bind {
                        var: "1".to_string(),
                        val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("id".to_string())))),
                        body: TermPtr::from_term(Term::App(
                            TermPtr::from_term(Term::Force("1".to_string())),
                            "0".to_string()
                        ))
                    })
                })
            }
        )
    }

    #[test]
    fn test3() {
        let src = "0 <> id 1 <> 2.";
        
        let mut cbpv = translate(parser::parse(src).unwrap());
        let term = cbpv.remove("main").unwrap();

        assert_eq!(
            term,
            Term::Choice(vec![
                TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Zero))),
                TermPtr::from_term(Term::Bind {
                    var: "0".to_string(),
                    val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero))))),
                    body: TermPtr::from_term(Term::Bind {
                        var: "1".to_string(),
                        val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("id".to_string())))),
                        body: TermPtr::from_term(Term::App(
                            TermPtr::from_term(Term::Force("1".to_string())),
                            "0".to_string()
                        ))
                    })
                }),
                TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero)))))))
            ])
        )
    }

    #[test]
    fn test4() {
        let src = "const :: a -> b -> a
const x y = x.";

        let mut cbpv = translate(parser::parse(src).unwrap());
        let term = cbpv.remove("const").unwrap();

        assert_eq!(
            term,
            Term::Thunk(TermPtr::from_term(Term::Lambda {
                arg: Arg::Ident("x".to_string()),
                body: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(Term::Lambda {
                    arg: Arg::Ident("y".to_string()),
                    body: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("x".to_string()))))
                })))))
            }))
        )
    }

    #[test]
    fn test5() {
        let src = "const :: a -> b -> a
const x y = x.
        
id :: a -> a
id x = let f = const x in f 1.";

        let mut cbpv = translate(parser::parse(src).unwrap());
        let term = cbpv.remove("id").unwrap();

        assert_eq!(
            term,
            Term::Thunk(TermPtr::from_term(Term::Lambda {
                arg: Arg::Ident("x".to_string()),
                body: TermPtr::from_term(Term::Bind {
                    var: "f".to_string(),
                    val: TermPtr::from_term(Term::Bind {
                        var: "0".to_string(),
                        val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("x".to_string())))),
                        body: TermPtr::from_term(Term::Bind {
                            var: "1".to_string(),
                            val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("const".to_string())))),
                            body: TermPtr::from_term(Term::App(
                                TermPtr::from_term(Term::Force("1".to_string())),
                                "0".to_string()
                            ))
                        })
                    }),
                    body: TermPtr::from_term(Term::Bind {
                        var: "0".to_string(),
                        val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero))))),
                        body: TermPtr::from_term(Term::Bind {
                            var: "1".to_string(),
                            val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Var("f".to_string())))),
                            body: TermPtr::from_term(Term::App(
                                TermPtr::from_term(Term::Force("1".to_string())),
                                "0".to_string()
                            ))
                        })
                    })
                })
            }))
        )
    }

    #[test]
    fn test6() {
        let src = "[1, 2, 3].";

        let mut cbpv = translate(parser::parse(src).unwrap());
        let term = cbpv.remove("main").unwrap();

        assert_eq!(
            term,
            Term::Bind {
                var: "0".to_string(),
                val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero))))),
                body: TermPtr::from_term(Term::Bind {
                    var: "1".to_string(),
                    val: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero))))))),
                    body: TermPtr::from_term(Term::Bind {
                        var: "2".to_string(),
                        val: TermPtr::from_term(Term::Return(TermPtr::from_term(
                            Term::Succ(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Succ(TermPtr::from_term(Term::Zero))))))
                        ))),
                        body: TermPtr::from_term(Term::Return(TermPtr::from_term(Term::Cons(
                            TermPtr::from_term(Term::Var("0".to_string())),
                            TermPtr::from_term(Term::Cons(
                                TermPtr::from_term(Term::Var("1".to_string())),
                                TermPtr::from_term(Term::Cons(
                                    TermPtr::from_term(Term::Var("2".to_string())),
                                    TermPtr::from_term(Term::Nil)
                                ))
                            ))
                        ))))
                    })
                })
            }
        )
    }
}