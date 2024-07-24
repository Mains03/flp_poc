use std::collections::{HashMap, HashSet};

use crate::parser::syntax::decl::Decl;

use super::term::Term;

pub trait Translate<'a> {
    fn translate(self, vars: &mut HashSet<String>, funcs: &mut HashMap<String, Decl<'a>>) -> Term<'a>;
}

#[cfg(test)]
mod test {
    use crate::parser::{self, syntax::r#type::Type};

    use super::*;

    #[test]
    fn test1() {
        let src = "const :: a -> b -> a
const x y = x.

const 1 2.";

        let cbpv = parser::parse(src).unwrap().translate(&mut HashSet::new(), &mut HashMap::new());

        assert_eq!(
            cbpv,
            Term::Bind {
                var: "0".to_string(),
                val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero))))))),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Bind {
                        var: "2".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))),
                        body: Box::new(Term::Bind {
                            var: "3".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                                args: vec!["y", "x"],
                                body: Box::new(Term::Return(Box::new(Term::Var("x".to_string()))))
                            }))))),
                            body: Box::new(Term::App(
                                Box::new(Term::Force(Box::new(Term::Var("3".to_string())))),
                                Box::new(Term::Var("2".to_string()))
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
        let src = "id :: Nat -> Nat
id x = exists n :: Nat. n =:= x. n.

let x = 1 in id x.";

        let cbpv = parser::parse(src).unwrap().translate(&mut HashSet::new(), &mut HashMap::new());

        assert_eq!(
            cbpv,
            Term::Bind {
                var: "x".to_string(),
                val: Box::new(Term::Return(
                    Box::new(Term::Succ(Box::new(Term::Zero)))
                )),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Return(
                        Box::new(Term::Var("x".to_string()))
                    )),
                    body: Box::new(Term::Bind {
                        var: "2".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                            args: vec!["x"],
                            body: Box::new(Term::Exists {
                                var: "n",
                                r#type: Type::Ident("Nat"),
                                body: Box::new(Term::Bind {
                                    var: "4".to_string(),
                                    val: Box::new(Term::Return(Box::new(Term::Var("n".to_string())))),
                                    body: Box::new(Term::Bind {
                                        var: "5".to_string(),
                                        val: Box::new(Term::Return(Box::new(Term::Var("x".to_string())))),
                                        body: Box::new(Term::Equate {
                                            lhs: Box::new(Term::Var("4".to_string())),
                                            rhs: Box::new(Term::Var("5".to_string())),
                                            body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
                                        })
                                    })
                                })
                            })
                        }))))),
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
    fn test3() {
        let src = "id :: a -> a
id x = x.

0 <> id 1 <> 2.";
        
        let cbpv = parser::parse(src).unwrap().translate(&mut HashSet::new(), &mut HashMap::new());

        assert_eq!(
            cbpv,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Zero)),
                Term::Bind {
                    var: "0".to_string(),
                    val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))),
                    body: Box::new(Term::Bind {
                        var: "1".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                            args: vec!["x"],
                            body: Box::new(Term::Return(Box::new(Term::Var("x".to_string()))))
                        }))))),
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
        let src = "addOne :: Nat -> Nat
addOne n = n + 1.

addOne 1.";

        let cbpv = parser::parse(src).unwrap().translate(&mut HashSet::new(), &mut HashMap::new());

        assert_eq!(
            cbpv,
            Term::Bind {
                var: "0".to_string(),
                val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                        args: vec!["n"],
                        body: Box::new(Term::Bind {
                            var: "3".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Var("n".to_string())))),
                            body: Box::new(Term::Bind {
                                var: "4".to_string(),
                                val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))),
                                body: Box::new(Term::Add(
                                    Box::new(Term::Var("3".to_string())),
                                    Box::new(Term::Var("4".to_string()))
                                ))
                            })
                        })
                    }))))),
                    body: Box::new(Term::App(
                        Box::new(Term::Force(Box::new(Term::Var("1".to_string())))),
                        Box::new(Term::Var("0".to_string()))
                    ))
                })
            }
        )
    }

    #[test]
    fn test5() {
        let src = "true.";

        let cbpv = parser::parse(src).unwrap().translate(&mut HashSet::new(), &mut HashMap::new());

        assert_eq!(
            cbpv,
            Term::Return(Box::new(Term::Bool(true)))
        );
    }

    #[test]
    fn test6() {
        let src = "if !(1 != 2) then 0 else 1.";

        let cbpv = parser::parse(src).unwrap().translate(&mut HashSet::new(), &mut HashMap::new());

        assert_eq!(
            cbpv,
            Term::Bind {
                var: "0".to_string(),
                val: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Bind {
                        var: "2".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))),
                        body: Box::new(Term::Bind {
                            var: "3".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero))))))),
                            body: Box::new(Term::NEq(
                                Box::new(Term::Var("2".to_string())),
                                Box::new(Term::Var("3".to_string()))
                            ))
                        })
                    }),
                    body: Box::new(Term::Not(Box::new(Term::Var("1".to_string()))))
                }),
                body: Box::new(Term::If {
                    cond: Box::new(Term::Var("0".to_string())),
                    then: Box::new(Term::Return(Box::new(Term::Zero))),
                    r#else: Box::new(Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))))
                })
            }
        );
    }
    
    #[test]
    fn test7() {
        let src = "id :: Nat -> Nat
id n = n.

exists n :: Nat. id n =:= id n. n.";

        let cbpv = parser::parse(src).unwrap().translate(&mut HashSet::new(), &mut HashMap::new());

        assert_eq!(
            cbpv,
            Term::Exists {
                var: "n",
                r#type: Type::Ident("Nat"),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(Term::Bind {
                        var: "3".to_string(),
                        val: Box::new(Term::Return(Box::new(Term::Var("n".to_string())))),
                        body: Box::new(Term::Bind {
                            var: "4".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                                args: vec!["n"],
                                body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
                            }))))),
                            body: Box::new(Term::App(
                                Box::new(Term::Force(Box::new(Term::Var("4".to_string())))),
                                Box::new(Term::Var("3".to_string()))
                            ))
                        })
                    }),
                    body: Box::new(Term::Bind {
                        var: "2".to_string(),
                        val: Box::new(Term::Bind {
                            var: "3".to_string(),
                            val: Box::new(Term::Return(Box::new(Term::Var("n".to_string())))),
                            body: Box::new(Term::Bind {
                                var: "4".to_string(),
                                val: Box::new(Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
                                    args: vec!["n"],
                                    body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
                                }))))),
                                body: Box::new(Term::App(
                                    Box::new(Term::Force(Box::new(Term::Var("4".to_string())))),
                                    Box::new(Term::Var("3".to_string()))
                                ))
                            })
                        }),
                        body: Box::new(Term::Equate {
                            lhs: Box::new(Term::Var("1".to_string())),
                            rhs: Box::new(Term::Var("2".to_string())),
                            body: Box::new(Term::Return(Box::new(Term::Var("n".to_string()))))
                        })
                    })
                })
            }
        );
    }
}