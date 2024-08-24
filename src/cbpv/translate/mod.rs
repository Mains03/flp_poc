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
    use std::collections::HashSet;

    use crate::parser;

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
                val: Box::new(Term::Return(Box::new(
                    Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero))))
                ))),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Bind {
                        var: "0".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))),
                        body: Box::new(Term::Bind {
                            var: "1".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Var("const".to_string())))),
                            body: Box::new(Term::App(
                                Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                                Box::new(Term::Var("0".to_string()))
                            ))
                        })
                    }),
                    body: Box::new(Term::App(
                        Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                        Box::new(Term::Var("0".to_string()))
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
                val: Box::new(Term::Return(
                    Box::new(Term::Succ(Box::new(Term::Zero)))
                )),
                body: Box::new(Term::Bind {
                    var: "0".to_string(),
                    val: Box::new(Term::Return(Box::new(Term::Var("x".to_string())))),
                    body: Box::new(Term::Bind {
                        var: "1".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Var("id".to_string())))),
                        body: Box::new(Term::App(
                            Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                            Box::new(Term::Var("0".to_string()))
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
                Term::Return(Box::new(Term::Zero)),
                Term::Bind {
                    var: "0".to_string(),
                    val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))),
                    body: Box::new(Term::Bind {
                        var: "1".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Var("id".to_string())))),
                        body: Box::new(Term::App(
                            Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                            Box::new(Term::Var("0".to_string()))
                        ))
                    })
                },
                Term::Return(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero))))))
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
            Term::Thunk(Box::new(Term::Lambda {
                var: "x".to_string(),
                free_vars: HashSet::new(),
                body: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                    var: "y".to_string(),
                    free_vars: HashSet::from_iter(vec!["x".to_string()]),
                    body: Box::new(Term::Return(Box::new(Term::Var("x".to_string()))))
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
            Term::Thunk(Box::new(Term::Lambda {
                var: "x".to_string(),
                free_vars: HashSet::from_iter(vec!["const".to_string()]),
                body: Box::new(Term::Bind {
                    var: "f".to_string(),
                    val: Box::new(Term::Bind {
                        var: "0".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Var("x".to_string())))),
                        body: Box::new(Term::Bind {
                            var: "1".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Var("const".to_string())))),
                            body: Box::new(Term::App(
                                Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                                Box::new(Term::Var("0".to_string()))
                            ))
                        })
                    }),
                    body: Box::new(Term::Bind {
                        var: "0".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))),
                        body: Box::new(Term::Bind {
                            var: "1".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Var("f".to_string())))),
                            body: Box::new(Term::App(
                                Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                                Box::new(Term::Var("0".to_string()))
                            ))
                        })
                    })
                })
            }))
        )
    }
}