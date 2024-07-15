use std::collections::HashMap;

use equate::eval_equate;
use exists::eval_exists;
use term::{substitute, Term};
use translate::translate;

use crate::parser::syntax::program::Decl;

pub mod term;
mod translate;
mod exists;
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
        Term::Succ(n1, v) => match v {
            Some(v) => if n1 == 0 {
                eval_term(*v, env)
            } else {
                match eval_term(*v, env) {
                    Term::Var(v) => Term::Succ(n1, Some(Box::new(Term::Var(v)))),
                    Term::Succ(n2, v) => Term::Succ(n1+n2, v),
                    _ => unreachable!()
                }
            },
            None => Term::Succ(n1, None)
        },
        Term::Bind { var, val, body } => {
            let val = eval_term(*val, env);

            match val {
                Term::Return(val) => substitute(*body, &var, &val),
                Term::Choice(v) => Term::Choice(
                    v.into_iter()
                        .map(|t| Term::Bind {
                            var: var.clone(),
                            val: Box::new(t),
                            body: body.clone()
                        }).collect()
                ),
                Term::Fail => Term::Fail,
                _ => Term::Bind { var, val: Box::new(val), body }
            }
        },
        Term::Add(lhs, rhs) => {
            let lhs = eval_term(*lhs, env);
            let rhs = eval_term(*rhs, env);

            add_terms(lhs, rhs)
        },
        Term::App(lhs, rhs) => {
            let lhs = eval_term(*lhs, env);

            match lhs {
                Term::Lambda { args, body } => apply(
                    Term::Lambda { args, body },
                    *rhs
                ),
                t => Term::App(Box::new(t), rhs)
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
            t => Term::Force(Box::new(t))
        },
        Term::Exists { var, r#type, body } => {
            eval_exists(
                var,
                r#type,
                eval_term(*body, env)
            )
        },
        Term::Equate { lhs, rhs, body } => {
            eval_equate(
                eval_term(*lhs, env),
                eval_term(*rhs, env),
                eval_term(*body, env)
            )
        },
        Term::Return(t) => Term::Return(Box::new(eval_term(*t, env))),
        t => t
    }
}

fn add_terms<'a>(lhs: Term<'a>, rhs: Term<'a>) -> Term<'a> {
    match lhs {
        Term::Succ(n1, ref v1) => match rhs {
            Term::Succ(n2, ref v2) => {
                match v1 {
                    Some(ref v1) => match v2 {
                        Some(_) => Term::Add(Box::new(lhs), Box::new(rhs)),
                        None => Term::Return(Box::new(Term::Succ(n1+n2, Some(v1.clone()))))
                    },
                    None => match v2 {
                        Some(ref v2) => Term::Return(Box::new(Term::Succ(n1+n2, Some(v2.clone())))),
                        None => Term::Return(Box::new(Term::Succ(n1+n2, None)))
                    }
                }
            },
            _ => Term::Add(Box::new(lhs), Box::new(rhs))
        },
        _ => Term::Add(Box::new(lhs), Box::new(rhs))
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
                Box::new(Term::Succ(5, None))
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
                Box::new(Term::Succ(1, None))
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
                Box::new(Term::Succ(5, None))
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
                Box::new(Term::Succ(5, None))
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
                Box::new(Term::Succ(1, None))
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
                Box::new(Term::Succ(1, None))
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
                Term::Return(Box::new(Term::Succ(1, None))),
                Term::Return(Box::new(Term::Succ(2, None)))
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
            Term::Return(Box::new(Term::Succ(1, None)))
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
                Term::Return(Box::new(Term::Succ(1, None))),
                Term::Return(Box::new(Term::Succ(2, None))),
                Term::Return(Box::new(Term::Succ(1, None))),
                Term::Return(Box::new(Term::Succ(2, None))),
                Term::Return(Box::new(Term::Succ(1, None))),
                Term::Return(Box::new(Term::Succ(1, None))),
                Term::Return(Box::new(Term::Succ(2, None))),
                Term::Return(Box::new(Term::Succ(2, None))),
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
                Term::Return(Box::new(Term::Succ(1, None))),
                Term::Return(Box::new(Term::Succ(1, None))),
                Term::Return(Box::new(Term::Succ(2, None))),
                Term::Return(Box::new(Term::Succ(2, None))),
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
            Term::Return(Box::new(Term::Succ(1, None))),
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
            Term::Return(Box::new(Term::Succ(1, None))),
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
            Term::Return(Box::new(Term::Succ(2, None))),
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
            Term::Return(Box::new(Term::Succ(2, None))),
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
            Term::Return(Box::new(Term::Succ(5, None))),
        );
    }

    #[test]
    fn test19() {
        let src = "exists n :: Nat. n + 1 =:= 5. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(4, None))),
        );
    }

    #[test]
    fn test20() {
        let src = "exists n :: Nat. n + n =:= 2. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(2, None))),
        );
    }
}