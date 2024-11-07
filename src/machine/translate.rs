use std::{collections::HashMap, rc::Rc};
use crate::{cbpv::terms::ValueType, parser::syntax::{bexpr::BExpr, expr::Expr, stm::Stm, r#type::Type}};
use super::mterms::{MComputation, MValue};

type Idx = usize;
struct Env { env : Vec<String> } 

impl Env {
    fn new() -> Env { Env { env: vec![] } }
    fn find(&self, v : &String) -> usize {
        self.env.iter().rev().position(|x| x == v).expect("Variable not found in environment")
    }
    fn bind(&mut self, v : &String) {
        self.env.push(v.clone())
    }
    fn unbind(&mut self) {
        self.env.pop();
    }
}

fn translate_vtype(ptype : Type) -> ValueType { 
    match ptype {
        Type::Arrow(_, _) => panic!("don't translate thunks"),
        Type::Ident(_) => todo!(),
        Type::List(t) => ValueType::List(Box::new(translate_vtype(*t))),
        Type::Pair(t1, t2) => ValueType::Pair(Box::new(translate_vtype(*t1)), Box::new(translate_vtype(*t2)))
    }
}

fn translate_stm(stm: Stm, env : &Env) -> MComputation {
    match stm {
        Stm::If { cond, then, r#else } => MComputation::Bind {
            comp : translate_stm(*cond, &env).into(),
            cont : todo!() // need sums to complete this
        },
        Stm::Let { var, val, body } => {
            let comp = translate_stm(*val, &env).into();
            env.bind(&var);
            let cont = translate_stm(*body, &env).into();
            env.unbind();
            MComputation::Bind { comp, cont }
        },
        Stm::Exists { var, r#type, body } => {
            env.bind(&var);
            let body: Rc<MComputation> = translate_stm(*body, env).into();
            env.unbind();
            let ptype = translate_vtype(r#type);
            MComputation::Exists { ptype, body: body }
        },
        Stm::Equate { lhs, rhs, body } => MComputation::Bind {
            comp: translate_expr(lhs, &env).into(),
            cont : MComputation::Bind {
                comp: translate_expr(rhs, &env).into(),
                cont: MComputation::Equate {
                    lhs : MValue::Var(1),
                    rhs : MValue::Var(0),
                    body : translate_stm(*body, env)
                }.into()
            }.into()
        },
        Stm::Choice(exprs) => MComputation::Choice(
            exprs.into_iter()
                .map(|e| translate_expr(e, &env).into()).collect()
        ),
        Stm::Case(var, case) => MComputation::PM(match case {
            Case::Nat(nat_case) => {
                let succ = nat_case.succ.unwrap();

                PM::PMNat(PMNat {
                    var,
                    zero: MComputationPtr::from_term(translate_expr(nat_case.zero.unwrap().expr)),
                    succ: PMNatSucc {
                        var: succ.var,
                        body: MComputationPtr::from_term(translate_expr(succ.expr))
                    }
                })
            },
            Case::List(list_case) => {
                let cons = list_case.cons.unwrap();

                PM::PMList(PMList {
                    var,
                    nil: MComputationPtr::from_term(translate_expr(list_case.empty.unwrap().expr)),
                    cons: PMListCons {
                        x: cons.x,
                        xs: cons.xs,
                        body: MComputationPtr::from_term(translate_expr(cons.expr))
                    }
                })
            }
        }),
        Stm::Expr(e) => translate_expr(e, &env)
    }
}

fn translate_expr(expr: Expr, env : &Env) -> MComputation {
    match expr {
        Expr::Cons(x, xs) => MComputation::Bind {
            var: "0".to_string(),
            val: MComputationPtr::from_term(translate_expr(*x)),
            body: MComputationPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: MComputationPtr::from_term(translate_expr(*xs)),
                body: MComputationPtr::from_term(Term::Return(TermPtr::from_term(Term::Cons(
                    MComputationPtr::from_term(Term::Var("0".to_string())),
                    MComputationPtr::from_term(Term::Var("1".to_string()))
                ))))
            })
        },
        Expr::Add(lhs, rhs) => MComputation::Bind {
            var: "0".to_string(),
            val: MComputationPtr::from_term(translate_expr(*lhs)),
            body: MComputationPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: MComputationPtr::from_term(translate_expr(*rhs)),
                body: MComputationPtr::from_term(Term::Add(
                    "0".to_string(),
                    "1".to_string()
                ))
            })
        },
        Expr::Lambda(arg, body) => {
            let body = translate_stm(*body);

            MComputation::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(
                MComputation::Lambda {
                    arg, body: MComputationPtr::from_term(body)
                }
            ))))
        },
        Expr::App(lhs, rhs) => MComputation::Bind {
            var: "0".to_string(),
            val: MComputationPtr::from_term(translate_expr(*rhs)),
            body: MComputationPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: MComputationPtr::from_term(translate_expr(*lhs)),
                body: MComputationPtr::from_term(Term::App(
                    MComputationPtr::from_term(Term::Force("1".to_string())),
                    "0".to_string()
                ))
            })
        },
        Expr::BExpr(bexpr) => translate_bexpr(bexpr),
        Expr::List(mut elems) => {
            elems.reverse();
            translate_list(elems, 0, vec![])
        },
        Expr::Ident(s) => MComputation::Return(TermPtr::from_term(Term::Var(s.clone()))),
        Expr::Nat(n) => MComputation::Return(TermPtr::from_term(translate_nat(n))),
        Expr::Bool(b) => MComputation::Return(TermPtr::from_term(Term::Bool(b))),
        Expr::Pair(lhs, rhs) => translate_pair(*lhs, *rhs),
        Expr::Stm(s) => translate_stm(*s)
    }
}

fn translate_bexpr(bexpr: BExpr, &env : Env) -> MComputation {
    match bexpr {
        BExpr::Eq(lhs, rhs) => MComputation::Bind {
            var: "0".to_string(),
            val: MComputationPtr::from_term(translate_expr(*lhs)),
            body: MComputationPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: MComputationPtr::from_term(translate_expr(*rhs)),
                body: MComputationPtr::from_term(Term::Eq(
                    "0".to_string(),
                    "1".to_string()
                ))
            })
        },
        BExpr::NEq(lhs, rhs) => MComputation::Bind {
            var: "0".to_string(),
            val: MComputationPtr::from_term(translate_expr(*lhs)),
            body: MComputationPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: MComputationPtr::from_term(translate_expr(*rhs)),
                body: MComputationPtr::from_term(Term::NEq(
                    "0".to_string(),
                    "1".to_string()
                ))
            })
        },
        BExpr::And(lhs, rhs) => MComputation::Bind {
            var: "0".to_string(),
            val: MComputationPtr::from_term(translate_expr(*lhs)),
            body: MComputationPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: MComputationPtr::from_term(translate_expr(*rhs)),
                body: MComputationPtr::from_term(Term::And(
                    MComputationPtr::from_term(Term::Var("0".to_string())),
                    MComputationPtr::from_term(Term::Var("1".to_string()))
                ))
            })
        },
        BExpr::Or(lhs, rhs) => MComputation::Bind {
            var: "0".to_string(),
            val: MComputationPtr::from_term(translate_expr(*lhs)),
            body: MComputationPtr::from_term(Term::Bind {
                var: "1".to_string(),
                val: MComputationPtr::from_term(translate_expr(*rhs)),
                body: MComputationPtr::from_term(Term::Or(
                    MComputationPtr::from_term(Term::Var("0".to_string())),
                    MComputationPtr::from_term(Term::Var("1".to_string()))
                ))
            })
        },
        BExpr::Not(e) => MComputation::Bind {
            var: "".to_string(),
            val: MComputationPtr::from_term(translate_expr(*e)),
            body: MComputationPtr::from_term(Term::Not("".to_string()))
        }
    }
}

fn translate_list(mut elems: Vec<Expr>, i: usize, mut list: Vec<MComputation>) -> Term {
    if elems.len() == 0 {
        list.reverse();

        MComputation::Return(TermPtr::from_term(
            list.into_iter()
                .fold(MComputation::Nil, |acc, t| {
                    MComputation::Cons(TermPtr::from_term(t), TermPtr::from_term(acc))
                })
        ))
    } else {
        let item = translate_expr(elems.remove(elems.len()-1));
        list.push(MComputation::Var(i.to_string()));

        MComputation::Bind {
            var: i.to_string(),
            val: MComputationPtr::from_term(item),
            body: MComputationPtr::from_term(
                translate_list(elems, i+1, list)
            )
        }
    }
}

fn translate_nat(n: usize) -> MComputation {
    if n == 0 {
        MComputation::Zero
    } else {
        MComputation::Succ(TermPtr::from_term(translate_nat(n-1)))
    }
}

fn translate_pair(lhs: Stm, rhs: Stm) -> MComputation {
    MComputation::Bind {
        var: "x".to_string(),
        val: MComputationPtr::from_term(translate_stm(lhs)),
        body: MComputationPtr::from_term(Term::Bind {
            var: "y".to_string(),
            val: MComputationPtr::from_term(translate_stm(rhs)),
            body: MComputationPtr::from_term(Term::Return(TermPtr::from_term(
                MComputation::Pair(
                    MComputationPtr::from_term(Term::Var("x".to_string())),
                    MComputationPtr::from_term(Term::Var("y".to_string()))
                )
            )))
        })
    }
}