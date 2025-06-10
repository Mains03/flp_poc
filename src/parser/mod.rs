use pest::{error::Error, Parser};
use pest_derive::Parser;

use crate::parser::{arg::Arg, bexpr::BExpr, cases::{Cases, CasesType}, decl::Decl, expr::Expr, stm::Stm, r#type::Type};

pub mod arg;
pub mod cases;
pub mod decl;
pub mod r#type;
pub mod stm;
pub mod expr;
pub mod bexpr;

#[derive(Parser)]
#[grammar = "parser/lang.pest"]
struct FLPParser;

pub fn parse(src: &str) -> Result<Vec<Decl>, Error<Rule>> {
    let mut prog = vec![];

    let pairs = FLPParser::parse(Rule::program, src)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::declaration => {
                let pair: pest::iterators::Pair<Rule> = pair.into_inner().next().unwrap();

                prog.push(match pair.as_rule() {
                    Rule::function_type => {
                        let mut pair = pair.into_inner();

                        let name = pair.next().unwrap().as_str();
                        let r#type = parse_type(pair.next().unwrap().into_inner());

                        Decl::FuncType { name: name.to_string(), r#type }
                    },
                    Rule::function => {
                        let mut pair = pair.into_inner();

                        let name = pair.next().unwrap().as_str();
                        
                        let mut args = vec![];
                        let body;
                        loop {
                            let pair = pair.next().unwrap();

                            match pair.as_rule() {
                                Rule::argument => args.push(parse_argument(pair.into_inner())),
                                _ => {
                                    body = pair;
                                    break;
                                }
                            }
                        };

                        let body = parse_statement(body.into_inner());

                        Decl::Func { name: name.to_string(), args, body }
                    },
                    Rule::statement => Decl::Stm(parse_statement(pair.into_inner())),
                    _ => unreachable!()
                })
            },
            _ => ()
        }
    }

    Ok(prog)
}

fn parse_argument(mut pairs: pest::iterators::Pairs<Rule>) -> Arg {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::identifier => Arg::Ident(pair.as_str().to_string()),
        Rule::argument_pair => parse_argument_pair(pair.into_inner()),
        _ => unreachable!()
    }
}

fn parse_argument_pair(mut pairs: pest::iterators::Pairs<Rule>) -> Arg {
    Arg::Pair(
        Box::new(parse_argument(pairs.next().unwrap().into_inner())),
        Box::new(parse_argument(pairs.next().unwrap().into_inner()))
    )
}

fn parse_statement(mut pairs: pest::iterators::Pairs<Rule>) -> Stm {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::_if => {
            let mut pairs = pair.into_inner();

            let cond = Box::new(parse_statement(pairs.next().unwrap().into_inner()));
            let then = Box::new(parse_statement(pairs.next().unwrap().into_inner()));
            let r#else = Box::new(parse_statement(pairs.next().unwrap().into_inner()));

            Stm::If { cond, then, r#else }
        },
        Rule::_let => {
            let mut pairs = pair.into_inner();

            let var = pairs.next().unwrap().as_str();
            let val = Box::new(parse_statement(pairs.next().unwrap().into_inner()));
            let body = Box::new(parse_statement(pairs.next().unwrap().into_inner()));

            Stm::Let { var: var.to_string(), val, body }
        },
        Rule::exists => {
            let mut pairs = pair.into_inner();

            let var = pairs.next().unwrap().as_str();
            let r#type = parse_type(pairs.next().unwrap().into_inner());
            let body = Box::new(parse_statement(pairs.next().unwrap().into_inner()));

            Stm::Exists { var: var.to_string(), r#type, body }
        },
        Rule::equate => {
            let mut pairs = pair.into_inner();

            let lhs = parse_expression(pairs.next().unwrap().into_inner());
            let rhs = parse_expression(pairs.next().unwrap().into_inner());
            let body = Box::new(parse_statement(pairs.next().unwrap().into_inner()));

            Stm::Equate { lhs, rhs, body }
        },
        Rule::choice => {
            let mut pairs = pair.into_inner();

            let mut choice = vec![];
            loop {
                match pairs.next() {
                    Some(p) => choice.push(parse_expression(p.into_inner())),
                    None => break
                }
            }

            Stm::Choice(choice)
        },
        Rule::case => {
            let mut pairs = pair.into_inner();

            let expr = parse_expression(pairs.next().unwrap().into_inner());
            let cases = parse_cases(pairs.next().unwrap().into_inner());

            Stm::Case { expr, cases }
        },
        Rule::expression => Stm::Expr(parse_expression(pair.into_inner())),
        _ => unreachable!()
    }
}

fn parse_expression(mut pairs: pest::iterators::Pairs<Rule>) -> Expr {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::cons => {
            let mut pairs = pair.into_inner();

            Expr::Cons(
                Box::new(parse_expression(pairs.next().unwrap().into_inner())),
                Box::new(parse_expression(pairs.next().unwrap().into_inner()))
            )
        },
        Rule::succ => {
            let mut pairs = pair.into_inner();

            Expr::Succ(Box::new(parse_expression(pairs.next().unwrap().into_inner())))
        },
        Rule::application => {
            let mut pairs = pair.into_inner();

            let mut exprs = vec![];
            loop {
                match pairs.next() {
                    Some(e) => exprs.push(parse_expression(e.into_inner())),
                    None => break,
                }
            }

            exprs.iter().fold(None, |acc, x| {
                match acc {
                    Some(e) => {
                        Some(Expr::App(
                            Box::new(e),
                            Box::new(x.clone())
                        ))
                    },
                    None => Some(x.clone())
                }
            }).unwrap()
        },
        Rule::boolean_expr => Expr::BExpr(parse_boolean_expression(pair.into_inner())),
        Rule::pair => {
            let mut pairs = pair.into_inner();

            Expr::Pair(
                Box::new(parse_expression(pairs.next().unwrap().into_inner())),
                Box::new(parse_expression(pairs.next().unwrap().into_inner()))
            )
        },
        Rule::list => Expr::List(parse_list(pair.into_inner())),
        Rule::lambda => {
            let mut pairs = pair.into_inner();
            Expr::Lambda(
                parse_argument(pairs.next().unwrap().into_inner()),
                Box::new(parse_statement(pairs.next().unwrap().into_inner()))
            )
        },
        Rule::primary_expr => parse_expression(pair.into_inner()),
        Rule::nat_zero => Expr::Zero,
        Rule::list_nil => Expr::Nil,
        Rule::identifier => Expr::Ident(pair.as_str().to_string()),
        Rule::number => Expr::Nat(pair.as_str().parse().unwrap()),
        Rule::boolean => Expr::Bool(parse_bool(pair.as_str())),
        Rule::statement => Expr::Stm(Box::new(parse_statement(pair.into_inner()))),
        _ => unreachable!()
    }
}

fn parse_cases(mut pairs: pest::iterators::Pairs<Rule>) -> Cases {
    let mut cases = Cases::new();

    loop {
        match pairs.next() {
            Some(p) => {
                let expr = parse_expression(p.into_inner());
                let body = parse_expression(pairs.next().unwrap().into_inner());

                match expr.strip_parentheses() {
                    Expr::Zero => {
                        cases.set_type_or_check(CasesType::Nat);
                        cases.set_nat_zero(body);
                    },
                    Expr::Succ(e) => {
                        let var = match *e {
                            Expr::Ident(s) => s,
                            _ => panic!("expected identifier")
                        };

                        cases.set_type_or_check(CasesType::Nat);
                        cases.set_nat_succ(var, body);
                    },
                    Expr::Nil => {
                        cases.set_type_or_check(CasesType::List);
                        cases.set_list_nil(body);
                    },
                    Expr::Cons(e1, e2) => {
                        let x = match *e1 {
                            Expr::Ident(s) => s,
                            _ => panic!("expected identifier")
                        };

                        let xs = match *e2 {
                            Expr::Ident(s) => s,
                            _ => panic!("expected identifier")
                        };

                        cases.set_type_or_check(CasesType::List);
                        cases.set_list_cons(x, xs, body);
                    },
                    _ => panic!("bad cases")
                }
            },
            None => break
        }
    }

    cases
}

fn parse_boolean_expression(mut pairs: pest::iterators::Pairs<Rule>) -> BExpr {
    let pair = pairs.next().unwrap();

    let lhs = parse_expression(pair.into_inner());

    let pair = pairs.next();
    match pair {
        Some(pair) => {
            let op = pair.as_str();

            let pair = pairs.next().unwrap();
            let rhs  = parse_expression(pair.into_inner());

            if op == "==" {
                BExpr::Eq(Box::new(lhs), Box::new(rhs))
            } else if op == "!=" {
                BExpr::NEq(Box::new(lhs), Box::new(rhs))
            } else if op == "&&" {
                BExpr::And(Box::new(lhs), Box::new(rhs))
            } else if op == "||" {
                BExpr::Or(Box::new(lhs), Box::new(rhs))
            } else {
                unreachable!()
            }
        },
        None => BExpr::Not(Box::new(lhs))
    }
}

fn parse_list(mut pairs: pest::iterators::Pairs<Rule>) -> Vec<Expr> {
    let mut list = vec![];

    loop {
        match pairs.next() {
            Some(pair) => list.push(parse_expression(pair.into_inner())),
            None => break
        }
    }

    list
}

fn parse_bool(s: &str) -> bool {
    if s == "true" {
        true
    } else if s == "false" {
        false
    } else {
        unreachable!("{:#?}", s);
    }
}

fn parse_type(mut pairs: pest::iterators::Pairs<Rule>) -> Type {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::arrow_type => parse_arrow_type(pair.into_inner()),
        Rule::primary_type => parse_primary_type(pair.into_inner()),
        t => unreachable!("{:#?}", t)
    }
}

fn parse_arrow_type(mut pairs: pest::iterators::Pairs<Rule>) -> Type {
    let lhs = parse_primary_type(pairs.next().unwrap().into_inner());
    let rhs = parse_type(pairs.next().unwrap().into_inner());

    Type::Arrow(Box::new(lhs), Box::new(rhs))
}

fn parse_primary_type(mut pairs: pest::iterators::Pairs<Rule>) -> Type {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::identifier => Type::Ident(pair.as_str().to_string()),
        Rule::list_type => Type::List(Box::new(parse_type(pair.into_inner().next().unwrap().into_inner()))),
        Rule::pair_type => {
            let mut pair = pair.into_inner();

            Type::Pair(
                Box::new(parse_type(pair.next().unwrap().into_inner())),
                Box::new(parse_type(pair.next().unwrap().into_inner()))
            )
        },
        Rule::r#type => parse_type(pair.into_inner()),
        _ => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let src = "const :: a -> b -> a
const x y = x.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::FuncType {
                    name: "const".to_string(),
                    r#type: Type::Arrow(
                        Box::new(Type::Ident("a".to_string())),
                        Box::new(Type::Arrow(
                            Box::new(Type::Ident("b".to_string())),
                            Box::new(Type::Ident("a".to_string()))
                        ))
                    )
                },
                Decl::Func {
                    name: "const".to_string(),
                    args: vec![Arg::Ident("x".to_string()), Arg::Ident("y".to_string())],
                    body: Stm::Expr(Expr::Ident("x".to_string()))
                }
            ]
        )
    }

    #[test]
    fn test2() {
        let src = "const :: a -> b -> a
const x y = x.

id :: a -> a
id x = x.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::FuncType {
                    name: "const".to_string(),
                    r#type: Type::Arrow(
                        Box::new(Type::Ident("a".to_string())),
                        Box::new(Type::Arrow(
                            Box::new(Type::Ident("b".to_string())),
                            Box::new(Type::Ident("a".to_string()))
                        ))
                    )
                },
                Decl::Func {
                    name: "const".to_string(),
                    args: vec![Arg::Ident("x".to_string()), Arg::Ident("y".to_string())],
                    body: Stm::Expr(Expr::Ident("x".to_string()))
                },
                Decl::FuncType {
                    name: "id".to_string(),
                    r#type: Type::Arrow(
                        Box::new(Type::Ident("a".to_string())),
                        Box::new(Type::Ident("a".to_string()))
                    )
                },
                Decl::Func {
                    name: "id".to_string(),
                    args: vec![Arg::Ident("x".to_string())],
                    body: Stm::Expr(Expr::Ident("x".to_string()))
                }
            ]
        )
    }

    #[test]
    fn test3() {
        let src = "fix :: (Nat -> Nat) -> Nat
fix f = exists n :: Nat. f n =:= n. n.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::FuncType {
                    name: "fix".to_string(),
                    r#type: Type::Arrow(
                        Box::new(Type::Arrow(
                            Box::new(Type::Ident("Nat".to_string())),
                            Box::new(Type::Ident("Nat".to_string()))
                        )),
                        Box::new(Type::Ident("Nat".to_string()))
                    )
                },
                Decl::Func {
                    name: "fix".to_string(),
                    args: vec![Arg::Ident("f".to_string())],
                    body: Stm::Exists {
                        var: "n".to_string(),
                        r#type: Type::Ident("Nat".to_string()),
                        body: Box::new(Stm::Equate {
                            lhs: Expr::App(
                                Box::new(Expr::Ident("f".to_string())),
                                Box::new(Expr::Ident("n".to_string()))
                            ),
                            rhs: Expr::Ident("n".to_string()),
                            body: Box::new(Stm::Expr(Expr::Ident("n".to_string())))
                        })
                    }
                }
            ]
        )
    }

    #[test]
    fn test4() {
        let src = "exists n :: Nat. n =:= 52. n.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::Stm(Stm::Exists {
                    var: "n".to_string(),
                    r#type: Type::Ident("Nat".to_string()),
                    body: Box::new(Stm::Equate {
                        lhs: Expr::Ident("n".to_string()),
                        rhs: Expr::Nat(52),
                        body: Box::new(Stm::Expr(Expr::Ident("n".to_string())))
                    })
                })
            ]
        )
    }

    #[test]
    fn test5() {
        let src: &str = "id :: Nat -> Nat
id x = exists n :: Nat. n =:= x. n.

id 5.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::FuncType { name: "id".to_string(), r#type: Type::Arrow(
                    Box::new(Type::Ident("Nat".to_string())),
                    Box::new(Type::Ident("Nat".to_string())))
                },
                Decl::Func {
                    name: "id".to_string(),
                    args: vec![Arg::Ident("x".to_string())],
                    body: Stm::Exists {
                        var: "n".to_string(),
                        r#type: Type::Ident("Nat".to_string()),
                        body: Box::new(Stm::Equate {
                            lhs: Expr::Ident("n".to_string()),
                            rhs: Expr::Ident("x".to_string()),
                            body: Box::new(Stm::Expr(Expr::Ident("n".to_string())))
                        })
                    }
                },
                Decl::Stm(Stm::Expr(Expr::App(
                    Box::new(Expr::Ident("id".to_string())),
                    Box::new(Expr::Nat(5))
                )))
            ]
        )
    }

    #[test]
    fn test6() {
        let src = "id x = x.

id 5.

id :: a -> a";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::Func {
                    name: "id".to_string(),
                    args: vec![Arg::Ident("x".to_string())],
                    body: Stm::Expr(Expr::Ident("x".to_string()))
                },
                Decl::Stm(Stm::Expr(Expr::App(
                    Box::new(Expr::Ident("id".to_string())),
                    Box::new(Expr::Nat(5))
                ))),
                Decl::FuncType {
                    name: "id".to_string(),
                    r#type: Type::Arrow(
                        Box::new(Type::Ident("a".to_string())),
                        Box::new(Type::Ident("a".to_string()))
                    )
                }
            ]
        );
    }

    #[test]
    fn test7() {
        let src = "id :: a -> a
id x = x.

1 <> id 2 <> 3.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::FuncType {
                    name: "id".to_string(),
                    r#type: Type::Arrow(
                        Box::new(Type::Ident("a".to_string())),
                        Box::new(Type::Ident("a".to_string())))
                    },
                Decl::Func {
                    name: "id".to_string(),
                    args: vec![Arg::Ident("x".to_string())],
                    body: Stm::Expr(Expr::Ident("x".to_string()))
                },
                Decl::Stm(Stm::Choice(vec![
                    Expr::Nat(1), Expr::App(Box::new(Expr::Ident("id".to_string())), Box::new(Expr::Nat(2))), Expr::Nat(3)
                ]))
            ]
        )
    }

    // #[test]
    // fn test8() {
    //     let src = "5 + 2.";

    //     let ast = parse(src).unwrap();

    //     assert_eq!(
    //         ast,
    //         vec![
    //             Decl::Stm(Stm::Expr(Expr::Add(
    //                 Box::new(Expr::Nat(5)),
    //                 Box::new(Expr::Nat(2))
    //             )))
    //         ]
    //     );
    // }

    // #[test]
    // fn test9() {
    //     let src = "1 + 2 + 3.";

    //     let ast = parse(src).unwrap();

    //     assert_eq!(
    //         ast,
    //         vec![
    //             Decl::Stm(Stm::Expr(Expr::Add(
    //                 Box::new(Expr::Nat(1)),
    //                 Box::new(Expr::Add(
    //                     Box::new(Expr::Nat(2)),
    //                     Box::new(Expr::Nat(3))
    //                 ))
    //             )))
    //         ]
    //     )
    // }

    #[test]
    fn test10() {
        let src = "true.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::Stm(Stm::Expr(Expr::Bool(true)))
            ]
        );
    }

    #[test]
    fn test11() {
        let src = "true == false.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::Stm(Stm::Expr(Expr::BExpr(BExpr::Eq(
                    Box::new(Expr::Bool(true)),
                    Box::new(Expr::Bool(false))
                ))))
            ]
        );
    }

    #[test]
    fn test12() {
        let src = "if !(1 != 2) then 0 else 1.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::Stm(Stm::If {
                    cond: Box::new(Stm::Expr(Expr::BExpr(BExpr::Not(Box::new(
                        Expr::Stm(Box::new(Stm::Expr(Expr::BExpr(BExpr::NEq(
                            Box::new(Expr::Nat(1)),
                            Box::new(Expr::Nat(2))
                        )))))
                    ))))),
                    then: Box::new(Stm::Expr(Expr::Nat(0))),
                    r#else: Box::new(Stm::Expr(Expr::Nat(1)))
                })
            ]
        );
    }
    
    #[test]
    fn test13() {
        let src = "exists xs :: [Nat]. xs =:= [1,2,3]. xs.";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![
                Decl::Stm(Stm::Exists {
                    var: "xs".to_string(),
                    r#type: Type::List(Box::new(Type::Ident("Nat".to_string()))),
                    body: Box::new(Stm::Equate {
                        lhs: Expr::Ident("xs".to_string()),
                        rhs: Expr::List(vec![Expr::Nat(1), Expr::Nat(2), Expr::Nat(3)]),
                        body: Box::new(Stm::Expr(Expr::Ident("xs".to_string())))
                    })
                })
            ]
        )
    }

    // #[test]
    // fn test14() {
    //     let src = "add_pair (x,y) = x+y.";

    //     let ast = parse(src).unwrap();

    //     assert_eq!(
    //         ast,
    //         vec![Decl::Func {
    //             name: "add_pair".to_string(),
    //             args: vec![Arg::Pair(
    //                 Box::new(Arg::Ident("x".to_string())),
    //                 Box::new(Arg::Ident("y".to_string()))
    //             )],
    //             body: Stm::Expr(Expr::Add(
    //                 Box::new(Expr::Ident("x".to_string())),
    //                 Box::new(Expr::Ident("y".to_string()))
    //             ))
    //         }]
    //     )
    // }

    // #[test]
    // fn test15() {
    //     let src = "add (x,(y,z)) = x+y+z.";

    //     let ast = parse(src).unwrap();

    //     assert_eq!(
    //         ast,
    //         vec![Decl::Func {
    //             name: "add".to_string(),
    //             args: vec![Arg::Pair(
    //                 Box::new(Arg::Ident("x".to_string())),
    //                 Box::new(Arg::Pair(
    //                     Box::new(Arg::Ident("y".to_string())),
    //                     Box::new(Arg::Ident("z".to_string()))
    //                 ))
    //             )],
    //             body: Stm::Expr(Expr::Add(
    //                 Box::new(Expr::Ident("x".to_string())),
    //                 Box::new(Expr::Add(
    //                     Box::new(Expr::Ident("y".to_string())),
    //                     Box::new(Expr::Ident("z".to_string()))
    //                 ))
    //             ))
    //         }]
    //     )
    // }

    #[test]
    fn test16() {
        let src = "pair :: a -> (b -> (a, b))";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![Decl::FuncType {
                name: "pair".to_string(),
                r#type: Type::Arrow(
                    Box::new(Type::Ident("a".to_string())),
                    Box::new(Type::Arrow(
                        Box::new(Type::Ident("b".to_string())),
                        Box::new(Type::Pair(
                            Box::new(Type::Ident("a".to_string())),
                            Box::new(Type::Ident("b".to_string()))
                        ))
                    ))
                )
            }]
        )
    }

    #[test]
    fn test17() {
        let src = "half :: [Nat] -> ([Nat], [Nat])";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![Decl::FuncType {
                name: "half".to_string(),
                r#type: Type::Arrow(
                    Box::new(Type::List(Box::new(Type::Ident("Nat".to_string())))),
                    Box::new(Type::Pair(
                        Box::new(Type::List(Box::new(Type::Ident("Nat".to_string())))),
                        Box::new(Type::List(Box::new(Type::Ident("Nat".to_string()))))
                    ))
                )
            }]
        )
    }

    // #[test]
    // fn test18() {
    //     let src = "length xs = case xs of [] -> 0. (x:xs) -> 1 + (length xs).";

    //     let ast = parse(src).unwrap();

    //     assert_eq!(
    //         ast,
    //         vec![Decl::Func {
    //             name: "length".to_string(),
    //             args: vec![Arg::Ident("xs".to_string())],
    //             body: Stm::Case("xs".to_string(), Case::List(ListCase {
    //                 empty: Some(ListEmptyCase { expr: Expr::Nat(0) }),
    //                 cons: Some(ListConsCase {
    //                     x: "x".to_string(),
    //                     xs: "xs".to_string(),
    //                     expr: Expr::Add(
    //                         Box::new(Expr::Nat(1)),
    //                         Box::new(Expr::Stm(Box::new(Stm::Expr(Expr::App(
    //                             Box::new(Expr::Ident("length".to_string())),
    //                             Box::new(Expr::Ident("xs".to_string()))
    //                         ))))
    //                     ))
    //                 })
    //             }))
    //         }]
    //     )
    // }

    #[test]
    fn test19() {
        let src = "1 : 2 : [3,4].";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![Decl::Stm(Stm::Expr(Expr::Cons(
                Box::new(Expr::Nat(1)),
                Box::new(Expr::Cons(
                    Box::new(Expr::Nat(2)),
                    Box::new(Expr::List(vec![
                        Expr::Nat(3), Expr::Nat(4)
                    ]))
                ))
            )))]
        )
    }

    #[test]
    fn test20() {
        let src = "true && (false || true).";

        let ast = parse(src).unwrap();

        assert_eq!(
            ast,
            vec![Decl::Stm(Stm::Expr(Expr::BExpr(BExpr::And(
                Box::new(Expr::Bool(true)),
                Box::new(Expr::Stm(Box::new(Stm::Expr(Expr::BExpr(BExpr::Or(
                    Box::new(Expr::Bool(false)),
                    Box::new(Expr::Bool(true))
                ))))))
            ))))]
        )
    }
}