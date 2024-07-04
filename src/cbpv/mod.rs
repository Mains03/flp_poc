use std::collections::HashMap;

use term::Term;
use translate::translate;

use crate::parser::syntax::program::Decl;

pub mod term;
mod translate;

pub fn eval<'a>(ast: Vec<Decl<'a>>) -> Term<'a> {
    let mut env = create_env(ast);

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
    match term {
        Term::Var(s) => {
            env.get(&s).unwrap().clone()
        },
        Term::Bind { var, val, body } => {
            let val = match *val {
                Term::Return(v) => eval_term(*v, env),
                _ => unreachable!()
            };

            env.insert(var, val);

            eval_term(*body, env)
        },
        Term::App(lhs, rhs) => {
            let lhs = match *lhs {
                Term::Force(t) => {
                    let t = eval_term(*t, env);
                
                    match t {
                        Term::Thunk(t) => eval_term(*t, env),
                        _ => unreachable!()
                    }
                },
                _ => unreachable!()
            };

            let rhs = eval_term(*rhs, env);

            apply(lhs, rhs)
        },
        t => t
    }
}

fn apply<'a>(lhs: Term<'a>, rhs: Term<'a>) -> Term<'a> {
    match lhs {
        Term::Lambda { mut args, body } => {
            let var = args.remove(args.len()-1);
            let body = substitute(*body, var, &rhs);

            if args.len() == 0 {
                body
            } else {
                Term::Lambda {
                    args,
                    body: Box::new(body)
                }
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
                val,
                body: if flag { body } else {
                    Box::new(substitute(*body, var, sub))
                }
            }
        }
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