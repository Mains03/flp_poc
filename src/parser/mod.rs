use pest::{error::Error, Parser};
use pest_derive::Parser;

use syntax::{arg::Arg, expr::Expr, program::{Decl, Prog}, stm::{Stm, Stms}, r#type::Type};

pub mod syntax;

#[derive(Parser)]
#[grammar = "parser/lang.pest"]
struct FLPParser;

pub fn parse(src: &str) -> Result<Prog, Error<Rule>> {
    let mut prog = vec![];

    let pairs = FLPParser::parse(Rule::program, src)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::decl => {
                let pair = pair.into_inner().next().unwrap();

                prog.push(match pair.as_rule() {
                    Rule::func_type => {
                        let mut pair = pair.into_inner();

                        let name = pair.next().unwrap().as_str();
                        let r#type = parse_type(pair.next().unwrap().into_inner());

                        Decl::FuncType { name, r#type }
                    },
                    Rule::func => {
                        let mut pair = pair.into_inner();

                        let name = pair.next().unwrap().as_str();
                        
                        let mut args = vec![];
                        let body;
                        loop {
                            let pair = pair.next().unwrap();

                            match pair.as_rule() {
                                Rule::arg => args.push(parse_arg(pair.into_inner())),
                                _ => {
                                    body = pair;
                                    break;
                                }
                            }
                        };

                        let body = parse_stms(body.into_inner());

                        Decl::Func { name, args, body }
                    },
                    Rule::stms => Decl::Stms(parse_stms(pair.into_inner())),
                    _ => unreachable!()
                })
            },
            _ => ()
        }
    }

    Ok(prog)
}

fn parse_arg(mut pairs: pest::iterators::Pairs<Rule>) -> Arg {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::ident => pair.as_str(),
        _ => unreachable!()
    }
}

fn parse_stms(mut pairs: pest::iterators::Pairs<Rule>) -> Stms {
    let mut stms = vec![];

    loop {
        let pair = pairs.next();
        match pair {
            Some(p) => stms.push(parse_stm(p.into_inner())),
            _ => break,
        }
    }

    stms
}

fn parse_stm(mut pairs: pest::iterators::Pairs<Rule>) -> Stm {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::if_stm => {
            let mut pairs = pair.into_inner();

            let cond = parse_stms(pairs.next().unwrap().into_inner());
            let then = parse_stms(pairs.next().unwrap().into_inner());
            let r#else = parse_stms(pairs.next().unwrap().into_inner());

            Stm::If { cond, then, r#else }
        },
        Rule::let_stm => {
            let mut pairs = pair.into_inner();

            let var = pairs.next().unwrap().as_str();
            let val = parse_stms(pairs.next().unwrap().into_inner());
            let body = parse_stms(pairs.next().unwrap().into_inner());

            Stm::Let { var, val, body }
        },
        Rule::exists_stm => {
            let mut pairs = pair.into_inner();

            let var = pairs.next().unwrap().as_str();
            let r#type = parse_type(pairs.next().unwrap().into_inner());
            let body = Box::new(parse_stm(pairs.next().unwrap().into_inner()));

            Stm::Exists { var, r#type, body }
        },
        Rule::equate_stm => {
            let mut pairs = pair.into_inner();

            let lhs = parse_expr(pairs.next().unwrap().into_inner());
            let rhs = parse_expr(pairs.next().unwrap().into_inner());
            let body = Box::new(parse_stm(pairs.next().unwrap().into_inner()));

            Stm::Equate { lhs, rhs, body }
        },
        Rule::choice_stm => {
            let mut pairs = pair.into_inner();

            let mut choice = vec![];
            loop {
                match pairs.next() {
                    Some(p) => choice.push(parse_expr(p.into_inner())),
                    None => break
                }
            }

            Stm::Choice(choice)
        },
        Rule::expr => Stm::Expr(parse_expr(pair.into_inner())),
        _ => unreachable!()
    }
}

fn parse_expr(mut pairs: pest::iterators::Pairs<Rule>) -> Expr {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::app => {
            let mut pairs = pair.into_inner();

            let mut exprs: Vec<Expr> = vec![];
            loop {
                match pairs.next() {
                    Some(e) => exprs.push(parse_expr(e.into_inner())),
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
        Rule::primary_expr => parse_expr(pair.into_inner()),
        Rule::ident => Expr::Ident(pair.as_str()),
        Rule::nat => Expr::Nat(pair.as_str().parse().unwrap()),
        Rule::stms => Expr::Stms(parse_stms(pair.into_inner())),
        _ => unreachable!()
    }
}

fn parse_type(mut pairs: pest::iterators::Pairs<Rule>) -> Type {
    let lhs = {
        let pair = pairs.next().unwrap().into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::ident => Type::Ident(pair.as_str()),
            Rule::r#type => parse_type(pair.into_inner()),
            _ => unreachable!()
        }
    };

    match pairs.next() {
        Some(t) => Type::Arrow(
            Box::new(lhs),
            Box::new(parse_type(t.into_inner()))
        ),
        None => lhs
    }
}