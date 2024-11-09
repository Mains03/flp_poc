use std::{collections::HashMap, rc::Rc};
use crate::{cbpv::terms::ValueType, parser::syntax::{arg::{self, Arg}, bexpr::BExpr, case::Case, decl::Decl, expr::Expr, stm::Stm, r#type::Type}};
use crate::machine::translate::Expr::Ident;
use super::{mterms::{MComputation, MValue}, Env, VClosure};

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

pub fn translate(ast: Vec<Decl>) -> (MComputation, Rc<Env>) {
    
    let mut env = Env::empty();
    let mut tenv = TEnv::new();
    let mut main = None;

    ast.into_iter()
        .for_each(|decl| match decl {
            Decl::FuncType { name: _, r#type: _ } => (),
            Decl::Func { name, args, body } => {
                let result : Rc<MValue> = translate_func(&name, args, body, &mut tenv).into();
                println!("[DEBUG] definition: {} = {}", name, *result);
                tenv.bind(&name);
                env = env.extend_clos(result.clone(), env.clone())
            },
            Decl::Stm(stm) => {
                let stmt = translate_stm(stm, &mut tenv);
                println!("[DEBUG] final stmt : {}", stmt);
                println!("[DEBUG] in env : {:?}", tenv.to_string());
                main = Some(stmt)
            }
        });
    (main.expect("empty program"), env)
}

fn translate_func(name : &String, args: Vec<Arg>, body: Stm, env : &mut TEnv) -> MValue {
    
    env.bind(name);

    let mut vars : Vec<String> = args.iter().map(|arg| match arg {
        Arg::Ident(var) => var.clone(),
        _ => todo!()
    }).collect();
    
    vars.iter().for_each(|s| env.bind(s));
    let mbody = translate_stm(body, env);
    vars.iter().for_each(|s| env.unbind());
    
    env.unbind();
    
    let mut c : MComputation = MComputation::Lambda { body : mbody.into()}.into();
    while vars.len() > 1 {
        c = MComputation::Lambda { body : MComputation::Return(MValue::Thunk(c.into()).into()).into() }.into();
        vars.pop();
    }
    MValue::Thunk(MComputation::Rec { body: c.into() }.into())
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
        Stm::Equate { lhs, rhs, body } => {
            let lhs_comp = translate_expr(lhs, env).into();
            env.bind(&"_foo".to_string());
            let rhs_comp = translate_expr(rhs, env).into();
            env.bind(&"_foo2".to_string());
            let body_comp = translate_stm(*body, env).into();
            env.unbind();
            env.unbind();
            MComputation::Bind {
                comp: lhs_comp,
                cont : MComputation::Bind {
                    comp: rhs_comp,
                    cont: MComputation::Equate {
                        lhs : MValue::Var(1).into(),
                        rhs : MValue::Var(0).into(),
                        body : body_comp
                    }.into()
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
                    let nilk = translate_expr(list_case.empty.unwrap().expr, env).into();
                    let case = list_case.cons.unwrap();
                    env.bind(&case.x); 
                    env.bind(&case.xs);
                    let consk = translate_expr(case.expr, env).into();
                    env.unbind(); 
                    env.unbind();
                    MComputation::Match { list: MValue::Var(env.find(&var)).into(), nilk, consk }
                }
        },
        Stm::Expr(e) => translate_expr(e, env)
    }
}

fn translate_expr(expr: Expr, env : &mut TEnv) -> MComputation {
    match expr {
        Expr::Cons(x, xs) => {
            let comp_head = translate_expr(*x, env).into();
            env.bind(&"_foo".to_string());
            let comp_tail = translate_expr(*xs, env).into();
            env.unbind();
            MComputation::Bind { 
                comp: comp_head,
                cont: MComputation::Bind { 
                    comp: comp_tail,
                    cont: MComputation::Return(MValue::Cons(MValue::Var(1).into(), MValue::Var(0).into()).into()).into(),
                }.into()
            }
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
        Expr::App(op, arg) => {
            let comp_op = translate_expr(*op, env).into();
            
            env.bind(&"_foo".to_string());
            let comp_arg = translate_expr(*arg, env).into();
            env.unbind();
            
            MComputation::Bind {
                comp : comp_op,
                cont : MComputation::Bind {
                    comp : comp_arg,
                    cont : MComputation::App { 
                        op: MComputation::Force(MValue::Var(1).into()).into(),
                        arg: MValue::Var(0).into()
                    }.into()
                }.into()
            }
        },
        Expr::BExpr(bexpr) => translate_bexpr(bexpr, env),
        Expr::List(mut elems) => translate_list(elems, env),
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

fn translate_list(elems: Vec<Expr>, env : &mut TEnv) -> MComputation {
    match elems.as_slice() {
        [] => MComputation::Return(MValue::Nil.into()),
        [head , tail@ ..] => {
            let chead = translate_expr(head.clone(), env);
            env.bind(&"_foo".to_string());
            let ctail = translate_list(tail.to_vec(), env);
            env.unbind();
            MComputation::Bind {
                comp: chead.into(), 
                cont: MComputation::Bind { 
                    comp: ctail.into(),
                    cont: MComputation::Return(MValue::Cons(MValue::Var(1).into(), MValue::Var(0).into()).into()).into() }.into()
            }
        }
    }
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