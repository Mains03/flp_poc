use std::collections::HashMap;

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
                Decl::Func { name, args: _, body: _ } => {
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

fn eval_term<'a>(term: Term<'a>, env: &mut HashMap<String, Term<'a>>) -> Term<'a> {
    println!("{:#?}", term);

    match term {
        Term::Var(s) => eval_term(env.get(&s).unwrap().clone(), env),
        Term::Bind { var, val, body } => {
            let val = match eval_term(*val, env) {
                Term::Return(t) => *t,
                _ => unreachable!()
            };

            let old = env.insert(var.clone(), val);

            let t = eval_term(*body, env);

            match old {
                Some(v) => { env.insert(var, v); },
                None => ()
            };

            t
        },
        Term::App(lhs, rhs) => {
            let lhs = eval_term(*lhs, env);

            eval_term(apply(lhs, *rhs), env)
        },
        Term::Force(t) => {
            let t = eval_term(*t, env);

            match t {
                Term::Thunk(t) => *t,
                _ => unreachable!()
            }
        },
        Term::Return(t) => Term::Return(Box::new(
            eval_term(*t, env)
        )),
        t => t
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
id x = x

let x = 5 in id x";

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
const x y = x

let x = 5 in const 1 x";

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
const x y = x

let x = 5 in const x 1";

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
        let src = "id :: a -> a
id x = x

f :: (a -> a) -> a -> a
f g x = g x

let x = 5 in f (f id) x";

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
const x y = x

let x = 1 in const x (let x = 2 in x)";

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
const x y = x

let x = 1 in const (let x = 2 in x) x";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Nat(2))
            )
        );
    }
}