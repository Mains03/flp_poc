use std::collections::HashMap;

use term::Term;
use translate::translate;

use crate::parser::syntax::program::Decl;

pub mod term;
mod translate;

pub fn eval<'a>(ast: &'a Vec<Decl<'a>>) -> Term<'a> {
    let mut env = create_env(ast);

    let main = env.get("main").unwrap().clone();
    println!("{:#?}", main);

    eval_term(main, &mut env)
}

fn create_env<'a>(ast: &'a Vec<Decl<'a>>) -> HashMap<String, Term<'a>> {
    let mut env = HashMap::new();

    ast.iter()
        .for_each(|decl| {
            match decl {
                Decl::Func { name, args, body } => {
                    env.insert(name.to_string(), translate(decl));
                },
                Decl::Stm(s) => {
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
            println!("{:#?}", lhs);

            let lhs = match *lhs {
                Term::Force(t) => match *t {
                    Term::Thunk(l) => *l,
                    Term::Var(s) => match env.get(&s).unwrap().clone() {
                        Term::Thunk(l) => *l,
                        _ => unreachable!()
                    }
                    _ => unreachable!()
                },
                _ => unreachable!()
            };

            apply(lhs, *rhs)
        },
        t => t
    }
}

fn apply<'a>(lhs: Term<'a>, rhs: Term<'a>) -> Term<'a> {
    match lhs {
        Term::Lambda { args, body } => {
            if args.len() == 1 {
                substitute(*body, args.get(0).unwrap(), rhs)
            } else {
                unimplemented!()
                //Term::Lambda {
                //    , body: () }
            }
        },
        _ => unreachable!()
    }
}

fn substitute<'a>(term: Term<'a>, var: &str, sub: Term<'a>) -> Term<'a> {
    unimplemented!()
}