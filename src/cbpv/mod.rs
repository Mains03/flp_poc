use std::collections::HashMap;

use term::Term;
use translate::translate;

use crate::parser::syntax::program::Decl;

pub mod term;
mod translate;

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
            _ => unreachable!()
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
        Term::Exists { var, r#type: _, body } => match eval_term(*body, env) {
            Term::Fail => Term::Fail,
            Term::Equate { lhs, rhs, body } => {
                let lhs_flag = term_contains_var(&*lhs, &var.to_string());
                let rhs_flag = term_contains_var(&*rhs, &var.to_string());

                if lhs_flag || rhs_flag {
                    unimplemented!()
                } else {
                    *body
                }
            },
            t => if term_contains_var(&t, &var.to_string()) {
                unimplemented!()
            } else {
                t
            }
        },
        Term::Equate { lhs, rhs, body } => {
            let lhs = eval_term(*lhs, env);
            let rhs = eval_term(*rhs, env);

            match lhs {
                Term::Nat(lhs_val) => match rhs {
                    Term::Nat(rhs_val) => if lhs_val == rhs_val {
                        *body
                    } else {
                        Term::Fail
                    },
                    Term::Var(s) => substitute(*body, &s, &Term::Nat(lhs_val)),
                    _ => Term::Equate {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        body
                    }
                },
                Term::Var(ref s) => match rhs {
                    Term::Nat(rhs_val) => substitute(*body, &s, &Term::Nat(rhs_val)),
                    _ => Term::Equate {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        body
                    }
                },
                _ => Term::Equate {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    body
                }
            }
        },
        t => t
    }
}

fn term_contains_var(term: &Term, var: &String) -> bool {
    fn term_contains_var_helper(term: &Term, var: &String) -> bool {
        match term {
            Term::Var(s) => s == var,
            Term::If { cond, then, r#else } => {
                term_contains_var_helper(*&cond, var)
                || term_contains_var_helper(*&then, var)
                || term_contains_var_helper(*&r#else, var)
            },
            Term::Bind { var: v, val, body } => {
                if v == var {
                    false
                } else {
                    term_contains_var_helper(*&val, var)
                    || term_contains_var_helper(*&body, var)
                }
            },
            Term::Exists { var: v, r#type: _, body } => {
                if v == var {
                    false
                } else {
                    term_contains_var_helper(*&body, var)
                }
            },
            Term::Equate { lhs, rhs, body } => {
                term_contains_var_helper(*&lhs, var)
                || term_contains_var_helper(*&rhs, var)
                || term_contains_var_helper(*&body, var)
            },
            Term::Lambda { args, body } => {
                if args.contains(&var.as_str()) {
                    false
                } else {
                    term_contains_var_helper(*&body, var)
                }
            },
            Term::Choice(v) => v.iter()
                .map(|t| term_contains_var_helper(t, var))
                .fold(false, |v, acc| v || acc),
            Term::Thunk(t) => term_contains_var_helper(*&t, var),
            Term::Return(t) => term_contains_var_helper(*&t, var),
            Term::Force(t) => term_contains_var_helper(*&t, var),
            Term::App(lhs, rhs) => {
                term_contains_var_helper(*&lhs, var)
                || term_contains_var_helper(*&rhs, var)
            },
            _ => false
        }
    }

    term_contains_var_helper(term, var)
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
        let src = "const :: a -> b -> a
const x y = x.
        
exists n :: Nat. exists m :: Nat. const n m =:= 1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(1)))
        );
    }

    #[test]
    fn test16() {
        let src = "choose_zero :: Nat -> Nat -> Nat
choose_zero x y = (x =:= 0. x) <> (y =:= 0. y).

choose_zero 0 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Nat(0)))
        );
    }

    #[test]
    fn test17() {
        let src = "choose_zero :: Nat -> Nat -> Nat
choose_zero x y = (x =:= 0. x) <> (y =:= 0. y).

choose_zero 1 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Fail
        );
    }
}