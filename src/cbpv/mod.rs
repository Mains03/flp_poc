use std::collections::HashMap;

use equate::eval_equate;
use term::Term;
use translate::translate;

use crate::parser::syntax::{program::Decl, r#type::Type};

pub mod term;
mod translate;
mod equate;

pub fn eval<'a>(ast: Vec<Decl<'a>>) -> Term<'a> {
    let env: HashMap<String, Term> = create_env(ast);

    let main = env.get("main").unwrap().clone();

    eval_term(main, &env)
}

fn create_env<'a>(ast: Vec<Decl<'a>>) -> HashMap<String, Term<'a>> {
    let mut env: HashMap<String, Term> = HashMap::new();

    ast.into_iter()
        .for_each(|decl| {
            match &decl {
                Decl::Func { name, args, body: _ } => if args.len() == 0 {
                    env.insert(name.to_string(), translate(decl));
                } else {
                    env.insert(name.to_string(), translate(decl));
                },
                Decl::Stm(_s) => {
                    env.insert("main".to_string(), translate(decl));
                }
                _ => ()
            }
        });

    env.insert("+".to_string(),
        Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
            args: vec!["x", "y"],
            body: Box::new(Term::Add(
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Var("y".to_string()))
            ))
        })))));

    env
}

fn eval_term<'a>(mut term: Term<'a>, env: &HashMap<String, Term<'a>>) -> Term<'a> {
    loop {
        let t = eval_step(term.clone(), env);

        if t == term {
            break
        } else {
            term = t;
        }
    }

    term
}

fn eval_step<'a>(term: Term<'a>, env: &HashMap<String, Term<'a>>) -> Term<'a> {
    match term {
        Term::Var(s) => match env.get(&s) {
            Some(v) => v.clone(),
            None => Term::Var(s) // free variable
        },
        Term::Bind { var, val, body } => match eval_term(*val, env) {
            Term::Return(val) => substitute(*body, &var, &val),
            Term::Choice(v) => Term::Choice(
                v.into_iter()
                    .map(|t| Term::Bind {
                        var: var.clone(),
                        val: Box::new(t),
                        body: body.clone()
                    }).collect()
            ),
            t => substitute(*body, &var, &t)
        },
        Term::Add(lhs, rhs) => {
            let lhs = eval_term(*lhs, env);
            let rhs = eval_term(*rhs, env);

            match lhs {
                Term::Nat(n1) => match rhs {
                    Term::Nat(n2) => Term::Return(Box::new(Term::Nat(n1+n2))),
                    _ => if n1 == 0 {
                        Term::Return(Box::new(rhs))
                    } else {
                        Term::Add(Box::new(lhs), Box::new(rhs))
                    }
                },
                _ => match rhs {
                    Term::Nat(n2) => if n2 == 0 {
                        Term::Return(Box::new(lhs))
                    } else {
                        Term::Add(Box::new(lhs), Box::new(rhs))
                    },
                    _ => Term::Add(Box::new(lhs), Box::new(rhs))
                }
            }
        },
        Term::App(lhs, rhs) => {
            let lhs = eval_term(*lhs, env);

            match lhs {
                Term::Lambda { args, body } => apply(
                    Term::Lambda { args, body },
                    *rhs
                ),
                _ => unreachable!()
            }
        },
        Term::Choice(mut v) => if v.len() == 0 {
            Term::Fail
        } else if v.len() == 1 {
            eval_step(v.remove(0), env)
        } else {
            Term::Choice(
                v.into_iter()
                    .flat_map(|t: Term| flat_eval_step(t, env).into_iter()
                        .flat_map(|t| if t == Term::Fail {
                            vec![]
                        } else {
                            vec![t]
                        }))
                    .collect()
            )
        },
        Term::Force(t) => match eval_term(*t, env) {
            Term::Thunk(t) => *t,
            _ => unreachable!()
        },
        Term::Exists { var, r#type, body } => {
            let body = eval_term(*body, env);

            match body {
                Term::Fail => Term::Fail,
                Term::Equate { lhs, rhs, body } => {
                    let lhs_flag = match *lhs {
                        Term::Var(ref s) => if s == var { true } else { false },
                        _ => false
                    };

                    let rhs_flag = match *rhs {
                        Term::Var(ref s) => if s == var { true } else { false },
                        _ => false
                    };
                    if lhs_flag {
                        if is_succ_of(var, &*rhs) {
                            Term::Fail
                        } else {
                            substitute(*body, var, &rhs)
                        }
                    } else if rhs_flag {
                        if is_succ_of(var, &*lhs) {
                            Term::Fail
                        } else {
                            substitute(*body, var, &lhs)
                        }
                    } else {
                        exists_enumerate(var, r#type, Term::Equate {
                            lhs: Box::new(*lhs), rhs: Box::new(*rhs), body
                        })
                    }
                },
                t => exists_enumerate(var, r#type, t)
            }
        },
        Term::Equate { lhs, rhs, body } => {
            eval_equate(
                eval_term(*lhs, env),
                eval_term(*rhs, env),
                eval_term(*body, env)
            )
        },
        t => t
    }
}

fn exists_enumerate<'a>(var: &'a str, r#type: Type<'a>, term: Term<'a>) -> Term<'a> {
    match r#type {
        Type::Ident(s) => if s == "Nat" {
            Term::Choice(vec![
                substitute(term.clone(), var, &Term::Nat(0)),
                Term::Exists { var, r#type,
                    body: Box::new(substitute(term, var, &Term::Add(
                        Box::new(Term::Var(var.to_string())),
                        Box::new(Term::Nat(1))
                    )))
                }
            ])
        } else {
            unimplemented!()
        },
        _ => unreachable!()
    }
}

fn is_succ_of(var: &str, term: &Term) -> bool {
    match term {
        Term::Add(lhs, rhs) => {
            let lhs_flag = match **lhs {
                Term::Nat(_) => (false, true), // non-zero
                Term::Var(ref s) => (s == var, false),
                _ => (false, false)
            };

            let rhs_flag = match **rhs {
                Term::Nat(_) => (false, true), // non-zero
                Term::Var(ref s) => (s == var, false),
                _ => (false, false)
            };

            (lhs_flag.0 && rhs_flag.1) || (lhs_flag.1 && rhs_flag.0)
        },
        _ => false
    }
}

fn flat_eval_step<'a>(term: Term<'a>, env: &HashMap<String, Term<'a>>) -> Vec<Term<'a>> {
    match term {
        Term::Choice(v) => v.into_iter()
            .flat_map(|t: Term| flat_eval_step(t, env))
            .collect(),
        t => vec![eval_step(t, env)]
    }
}

fn apply<'a>(lhs: Term<'a>, rhs: Term<'a>) -> Term<'a> {
    match lhs {
        Term::Lambda { mut args, body } => {
            let var = args.remove(args.len()-1);
            let body: Term = substitute(*body, var, &rhs);

            if args.len() == 0 {
                body
            } else {
                Term::Return(Box::new(
                    Term::Thunk(Box::new(
                        Term::Lambda {
                            args,
                            body: Box::new(body)
                        }
                    ))
                ))
            }
        },
        _ => unreachable!()
    }
}

fn substitute<'a>(term: Term<'a>, var: &str, sub: &Term<'a>) -> Term<'a> {
    match term {
        Term::Var(s) => if s == var { sub.clone() } else { Term::Var(s) },
        Term::If { cond, then, r#else } => Term::If {
            cond: Box::new(substitute(*cond, var, sub)),
            then: Box::new(substitute(*then, var, sub)),
            r#else: Box::new(substitute(*r#else, var, sub))
        },
        Term::Bind { var: v, val, body } => {
            let flag = v == var;

            Term::Bind {
                var: v,
                val: Box::new(substitute(*val, var, sub)),
                body: if flag { body } else {
                    Box::new(substitute(*body, var, sub))
                }
            }
        },
        Term::Exists { var: v, r#type, body } => {
            Term::Exists {
                var: v,
                r#type,
                body: if v == var { body } else {
                    Box::new(substitute(*body, var, sub))
                }
            }
        },
        Term::Equate { lhs, rhs, body } => {
            Term::Equate {
                lhs: Box::new(substitute(*lhs, var, sub)),
                rhs: Box::new(substitute(*rhs, var, sub)),
                body: Box::new(substitute(*body, var, sub))
            }
        },
        Term::Lambda { args, body } => {
            let flag = args.contains(&var);

            Term::Lambda {
                args,
                body: if flag { 
                    body
                } else {
                    Box::new(substitute(*body, var, sub))
                }
            }
        },
        Term::Choice(c) => Term::Choice(
            c.into_iter()
                .map(|t| substitute(t, var, sub))
                .collect()
        ),
        Term::Thunk(t) => Term::Thunk(
            Box::new(substitute(*t, var, sub))
        ),
        Term::Return(t) => Term::Return(
            Box::new(substitute(*t, var, sub))
        ),
        Term::Force(t) => Term::Force(
            Box::new(substitute(*t, var, sub))
        ),
        Term::Add(lhs, rhs) => Term::Add(
            Box::new(substitute(*lhs, var, sub)),
            Box::new(substitute(*rhs, var, sub))
        ),
        Term::App(lhs, rhs) => Term::App(
            Box::new(substitute(*lhs, var, sub)),
            Box::new(substitute(*rhs, var, sub))
        ),
        t => t
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;

    use super::*;

    #[test]
    fn test1() {
        let src = "id :: a -> a
id x = x.

id 5.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Nat(5))
            )
        );
    }

    #[test]
    fn test2() {
        let src = "const :: a -> b -> a
const x y = x.

const 1 5.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Nat(1))
            )
        );
    }

    #[test]
    fn test3() {
        let src = "const :: a -> b -> a
const x y = x.

const 5 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Nat(5))
            )
        );
    }

    #[test]
    fn test4() {
        let src: &str = "id :: a -> a
id x = x.

f :: (a -> a) -> a -> a
f g x = g x.

f (f id) 5.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Nat(5))
            )
        );
    }

    #[test]
    fn test5() {
        let src = "const :: a -> b -> a
const x y = x.

const (let x = 1 in x) 2.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Nat(1))
            )
        )
    }

    #[test]
    fn test6() {
        let src = "const :: a -> b -> a
const x y = x.

let x = 1 in const x (let x = 2 in x).";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Nat(1))
            )
        );
    }

    #[test]
    fn test7() {
        let src = "const :: a -> b -> a
const x y = x.

const x (let x = 1 in x).";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Var("x".to_string())))
        )
    }

    #[test]
    fn test8() {
        let src = "const1 :: a -> b -> a
const1 x y = x.

const2 :: a -> b -> b
const2 x y = y.

let f = const1 <> const2 in f 1 2.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Nat(1))),
                Term::Return(Box::new(Term::Nat(2)))
            ])
        );
    }

    #[test]
    fn test9() {
        let src = "num :: Nat
num = 1.

num.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(1)))
        );
    }

    #[test]
    fn test10() {
        let src = "num :: Nat
num = 1 <> 2.

const1 :: Nat -> Nat -> Nat
const1 x y = x.

const2 :: Nat -> Nat -> Nat
const2 x y = y.

let f = const1 <> const2 in f num num.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Nat(1))),
                Term::Return(Box::new(Term::Nat(2))),
                Term::Return(Box::new(Term::Nat(1))),
                Term::Return(Box::new(Term::Nat(2))),
                Term::Return(Box::new(Term::Nat(1))),
                Term::Return(Box::new(Term::Nat(1))),
                Term::Return(Box::new(Term::Nat(2))),
                Term::Return(Box::new(Term::Nat(2))),
            ])
        )
    }

    #[test]
    fn test11() {
        let src = "f :: Nat -> Nat -> Nat
f = const1 <> const2.

const1 :: Nat -> Nat -> Nat
const1 x y = x.

const2 :: Nat -> Nat -> Nat
const2 x y = y.

let num = 1 <> 2 in f num num.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Nat(1))),
                Term::Return(Box::new(Term::Nat(1))),
                Term::Return(Box::new(Term::Nat(2))),
                Term::Return(Box::new(Term::Nat(2))),
            ])
        )
    }

    #[test]
    fn test12() {
        let src = "exists n :: Nat. n =:= 1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(1)))
        );
    }

    #[test]
    fn test13() {
        let src = "id :: a -> a
id x = x.
        
exists n :: Nat. id n =:= 1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(1)))
        );
    }

    #[test]
    fn test14() {
        let src = "exists n :: Nat. 0 =:= 1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Fail
        );
    }

    #[test]
    fn test15() {
        let src = "1 + 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(2)))
        );
    }

    #[test]
    fn test16() {
        let src = "addOne :: Nat -> Nat
addOne n = n + 1.

addOne 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(2)))
        );
    }

    #[test]
    fn test17() {
        let src: &str = "exists n :: Nat. n =:= n+1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val, 
            Term::Fail
        );
    }

    #[test]
    fn test18() {
        let src = "id :: Nat -> Nat
id n = exists m :: Nat. m =:= n. m.

id 5.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(5)))
        );
    }

    #[test]
    fn test19() {
        let src = "exists n :: Nat. n + 1 =:= 5. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(4)))
        );
    }

    #[test]
    fn test20() {
        let src = "exists n :: Nat. n + n =:= 2. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(2)))
        );
    }
}