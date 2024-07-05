use std::{collections::HashMap, io::stdin};

use term::Term;
use translate::translate;

use crate::parser::syntax::program::Decl;

pub mod term;
mod translate;

pub fn eval<'a>(ast: Vec<Decl<'a>>) -> Term<'a> {
    let mut env: HashMap<String, Term> = create_env(ast);

    let main = env.get("main").unwrap().clone();

    eval_term(main, &mut env)
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

    env
}

fn eval_term<'a>(mut term: Term<'a>, env: &mut HashMap<String, Term<'a>>) -> Term<'a> {
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

fn eval_step<'a>(term: Term<'a>, env: &mut HashMap<String, Term<'a>>) -> Term<'a> {
    match term {
        Term::Var(s) => env.get(&s).unwrap().clone(),
        Term::Bind { var, val, body } => match eval_step(*val, env) {
            Term::Return(val) => {
                let val = eval_step(*val, env);
                let old: Option<Term<'a>> = env.insert(var.clone(), val);

                let t = eval_step(*body, env);
    
                match old {
                    Some(v) => { env.insert(var.clone(), v); },
                    None => { env.remove(&var); }
                }
    
                t
            },
            Term::Choice(v) => Term::Choice(
                v.into_iter()
                    .map(|t| eval_step(
                        Term::Bind {
                            var: var.clone(),
                            val: Box::new(t),
                            body: body.clone()
                        },
                        env
                    )).collect()
            ),
            _ => unreachable!()
        },
        Term::App(lhs, rhs) => {
            let lhs = eval_step(*lhs, env);

            let rhs = eval_step(*rhs, env);

            match lhs {
                Term::Lambda { args, body } => apply(
                    Term::Lambda { args, body },
                    rhs
                ),
                t => Term::App(
                    Box::new(t),
                    Box::new(rhs)
                )
            }
        },
        Term::Choice(v) => Term::Choice(
            v.into_iter()
                .flat_map(|t: Term| flat_eval_step(t, env))
                .collect()
        ),
        Term::Lambda { args, body } => {
            let mut old = HashMap::new();

            args.iter()
                .for_each(|a| {
                    old.insert(
                        a,
                        env.insert(
                            a.to_string(),
                            Term::Var(a.to_string())
                        )
                    );
                });

            let t = Term::Lambda {
                args: args.clone(),
                body: Box::new(eval_step(*body, env))
            };

            old.into_iter()
                .for_each(|(a, t)| {
                    match t {
                        Some(v) => env.insert(
                            a.to_string(),
                            v
                        ),
                        None => env.remove(*a)
                    };
                });

            t
        },
        Term::Force(t) => match eval_step(*t, env) {
            Term::Thunk(t) => *t,
            t => Term::Force(Box::new(t))
        },
        Term::Thunk(t) => Term::Thunk(Box::new(eval_step(*t, env))),
        Term::Return(t) => Term::Return(Box::new(eval_step(*t, env))),
        t => t
    }
}

fn flat_eval_step<'a>(term: Term<'a>, env: &mut HashMap<String, Term<'a>>) -> Vec<Term<'a>> {
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
                body: if flag { body } else {
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
    #[should_panic]
    fn test7() {
        let src = "const :: a -> b -> a
const x y = x.

const x (let x = 1 in x).";

        let ast = parser::parse(src).unwrap();
        eval(ast);
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
}