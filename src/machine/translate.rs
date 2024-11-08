use std::{collections::HashMap, rc::Rc};
use crate::{cbpv::terms::ValueType, parser::syntax::{arg::{self, Arg}, bexpr::BExpr, case::Case, decl::Decl, expr::Expr, stm::Stm, r#type::Type}};
use crate::machine::translate::Expr::Ident;
use super::{empty_env, mterms::{MComputation, MValue}, Env, VClosure};

type Idx = usize;
struct TEnv { env : Vec<String> } 

impl TEnv {
    fn new() -> TEnv { TEnv { env: vec![] } }
    fn find(&self, v : &String) -> usize {
        self.env.iter().rev().position(|x| x == v).expect(&("Variable ".to_owned() + v + " not found in environment"))
    }
    fn bind(&mut self, v : &String) {
        self.env.push(v.clone())
    }
    fn unbind(&mut self) {
        self.env.pop();
    }
    fn to_string(&self) -> String {
        "[ ".to_owned() + &self.env.join(" ") + " ]"
    }
}

pub fn translate(ast: Vec<Decl>) -> (MComputation, Env) {
    
    let mut env = vec![];
    let mut tenv = TEnv::new();
    let mut main = None;

    ast.into_iter()
        .for_each(|decl| match decl {
            Decl::FuncType { name: _, r#type: _ } => (),
            Decl::Func { name, args, body } => {
                let result : Rc<MValue> = translate_func(args, body, &mut tenv).into();
                println!("[DEBUG] definition name : {}, body : {:?}", name, *result);
                tenv.bind(&name);
                let vclos = VClosure::Clos { val : result.clone(), env: env.clone().into() };
                env.push(vclos);
            },
            Decl::Stm(stm) => {
                let stmt = translate_stm(stm, &mut tenv);
                println!("[DEBUG] final stmt : {:?}", stmt);
                println!("[DEBUG] in env : {:?}", tenv.to_string());
                main = Some(stmt)
            }
        });
    (main.expect("empty program"), env)
}

fn translate_func(args: Vec<Arg>, body: Stm, env : &mut TEnv) -> MValue {

    let mut vars : Vec<String> = args.iter().map(|arg| match arg {
        Arg::Ident(var) => var.clone(),
        _ => todo!()
    }).collect();
    
    vars.iter().for_each(|s| env.bind(s));
    let mbody = translate_stm(body, env);
    vars.iter().for_each(|s| env.unbind());
    
    let mut mval = MValue::Thunk(MComputation::Lambda { body : mbody.into()}.into());
    while vars.len() > 1 {
        mval = MValue::Thunk(MComputation::Lambda { body : MComputation::Return(mval.into()).into() }.into());
        vars.pop();
    }
    mval 
}

fn translate_vtype(ptype : Type) -> ValueType { 
    match ptype {
        Type::Arrow(_, _) => panic!("don't translate thunks"),
        Type::Ident(s) => 
            if s == "Nat" { ValueType::Nat } else { todo!() },
        Type::List(t) => ValueType::List(Box::new(translate_vtype(*t))),
        Type::Pair(t1, t2) => ValueType::Pair(Box::new(translate_vtype(*t1)), Box::new(translate_vtype(*t2)))
    }
}

fn translate_stm(stm: Stm, env : &mut TEnv) -> MComputation {
    match stm {
        Stm::If { cond, then, r#else } => MComputation::Bind {
            comp : translate_stm(*cond, env).into(),
            cont : todo!() // need sums to complete this
        },
        Stm::Let { var, val, body } => {
            let comp = translate_stm(*val, env).into();
            env.bind(&var);
            let cont = translate_stm(*body, env).into();
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
            comp: translate_expr(lhs, env).into(),
            cont : MComputation::Bind {
                comp: translate_expr(rhs, env).into(),
                cont: MComputation::Equate {
                    lhs : MValue::Var(1).into(),
                    rhs : MValue::Var(0).into(),
                    body : translate_stm(*body, env).into()
                }.into()
            }.into()
        },
        Stm::Choice(exprs) => MComputation::Choice(
            exprs.into_iter()
                .map(|e| translate_expr(e, env).into()).collect()
        ),
        Stm::Case(var, case) => 
            match case {
                Case::Nat(nat_case) => {
                    MComputation::Ifz { 
                        num: MValue::Var(env.find(&var)).into(),
                        zk: translate_expr(nat_case.zero.unwrap().expr, env).into(),
                        sk: translate_expr(nat_case.succ.unwrap().expr, env).into(),
                    }
                },
                Case::List(list_case) => {
                    MComputation::Match { 
                        list: MValue::Var(env.find(&var)).into(),
                        nilk: translate_expr(list_case.empty.unwrap().expr, env).into(),
                        consk: translate_expr(list_case.cons.unwrap().expr, env).into(),
                    }
                }
        },
        Stm::Expr(e) => translate_expr(e, env)
    }
}

fn translate_expr(expr: Expr, env : &mut TEnv) -> MComputation {
    match expr {
        Expr::Cons(x, xs) => 
            MComputation::Bind { 
                comp: translate_expr(*x, env).into(),
                cont: MComputation::Bind { 
                    comp: translate_expr(*xs, env).into(), 
                    cont: MComputation::Return(MValue::Cons(MValue::Var(1).into(), MValue::Var(0).into()).into()).into(),
                }.into()
            },
        Expr::Add(arg1, arg2) =>
            MComputation::Bind { 
                comp: translate_expr(*arg1, env).into(),
                cont: MComputation::Bind { 
                    comp: translate_expr(*arg2, env).into(), 
                    cont: todo!(),
                }.into()
            },
        Expr::Lambda(arg, body) => {
            match arg {
                arg::Arg::Ident(var) => {
                    env.bind(&var);
                    let body = translate_stm(*body, env);
                    env.unbind();

                    MComputation::Return(MValue::Thunk(MComputation::Lambda { body: body.into() }.into()).into())
                },
                arg::Arg::Pair(arg, arg1) => todo!(),
            }
        },
        Expr::App(op, arg) => MComputation::Bind {
            comp : translate_expr(*op, env).into(),
            cont : MComputation::Bind {
                comp : translate_expr(*arg, env).into(),
                cont : MComputation::App { 
                    op: MComputation::Force(MValue::Var(1).into()).into(),
                    arg: MValue::Var(0).into()
                }.into()
            }.into()
        },
        Expr::BExpr(bexpr) => translate_bexpr(bexpr, env),
        Expr::List(mut elems) => {
            elems.reverse();
            translate_list(elems, 0, vec![])
        },
        Expr::Ident(s) => MComputation::Return(MValue::Var(env.find(&s)).into()),
        Expr::Nat(n) => translate_nat(n),
        Expr::Bool(b) => todo!("no bools yet"),
        Expr::Pair(lhs, rhs) => translate_pair(*lhs, *rhs, env),
        Expr::Stm(s) => translate_stm(*s, env)
    }
}

fn translate_bexpr(bexpr: BExpr, env : &TEnv) -> MComputation {
    match bexpr {
        BExpr::Eq(lhs, rhs) => todo!(),
        BExpr::NEq(lhs, rhs) => todo!(),
        BExpr::And(lhs, rhs) => todo!(),
        BExpr::Or(lhs, rhs) => todo!(),
        BExpr::Not(e) => todo!()
    }
}

fn translate_list(mut elems: Vec<Expr>, i: usize, mut list: Vec<MComputation>) -> MComputation {
    todo!("NOT YET DONE")
    // if elems.len() == 0 {
    //     list.reverse();

    //     MComputation::Return(
    //         list.into_iter()
    //             .fold(MComputation::Nil, |acc, t| {
    //                 MValue::Cons(TermPtr::from_term(t), TermPtr::from_term(acc))
    //             })
    //     )
    // } else {
    //     let item = translate_expr(elems.remove(elems.len()-1));
    //     list.push(MComputation::Var(i.to_string()));

    //     MComputation::Bind {
    //         var: i.to_string(),
    //         val: MComputationPtr::from_term(item),
    //         body: MComputationPtr::from_term(
    //             translate_list(elems, i+1, list)
    //         )
    //     }
    // }
}

fn translate_nat(n: usize) -> MComputation {
    let mut nat_val = MValue::Zero.into();
    for i in (0..n) {
        nat_val = MValue::Succ(nat_val).into();
    }
    MComputation::Return(nat_val.into())
}

fn translate_pair(fst: Stm, snd: Stm, env : &mut TEnv) -> MComputation {
    MComputation::Bind { 
        comp: translate_stm(fst, env).into(), 
        cont: MComputation::Bind {
            comp : translate_stm(snd, env).into(),
            cont : MComputation::Return(MValue::Pair(MValue::Var(1).into(), MValue::Var(0).into()).into()).into()
        }.into()
    }.into()
}