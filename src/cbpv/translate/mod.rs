use std::collections::HashMap;

use crate::parser::syntax::decl::Decl;

use super::Term;

mod decl;
mod stm;
mod expr;
mod bexpr;

pub fn translate(ast: Vec<Decl>) -> HashMap<String, Term> {
    let mut cbpv = HashMap::new();

    ast.into_iter()
        .for_each(|decl| match decl {
            Decl::FuncType { name: _, r#type: _ } => (),
            Decl::Func { name, args, body } => {
                cbpv.insert(
                    name,
                    Decl::Func { name: "".to_string(), args, body }.translate()
                );
            },
            Decl::Stm(stm) => {
                cbpv.insert("main".to_string(), stm.translate());
            }
        });

    cbpv
}

trait Translate {
    fn translate(self) -> Term;
}

#[cfg(test)]
mod test {
    use crate::{cbpv::term_ptr::TermPtr, parser::{self, syntax::arg::Arg}};

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