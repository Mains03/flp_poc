use std::collections::{HashMap, HashSet};

use crate::cbpv::{term::Term, translate::Translate};

use super::{r#type::Type, arg::Arg, stm::*};

pub type Prog<'a> = Vec<Decl<'a>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Decl<'a> {
    FuncType {
        name: &'a str,
        r#type: Type<'a>
    },
    Func {
        name: &'a str,
        args: Vec<Arg<'a>>,
        body: Stm<'a>
    },
    Stm(Stm<'a>)
}

impl<'a> Translate<'a> for Prog<'a> {
    fn translate(self, vars: &mut HashSet<String>, funcs: &mut HashMap<String, Decl<'a>>) -> Term<'a> {
        self.into_iter()
            .for_each(|decl| match decl {
                Decl::FuncType { name: _, r#type: _ } => (),
                Decl::Func { name, args, body } => { funcs.insert(
                    name.to_string(),
                    Decl::Func { name, args, body }
                ); },
                Decl::Stm(stm) => { funcs.insert("main".to_string(), Decl::Stm(stm)); }
            });

        funcs.remove("main").unwrap().translate(vars, funcs)
    }
}

impl<'a> Translate<'a> for Decl<'a> {
    fn translate(self, vars: &mut HashSet<String>, funcs: &mut HashMap<String, Decl<'a>>) -> Term<'a> {
        match self {
            Decl::Func { name: _, mut args, body } => {
                args.iter()
                    .for_each(|s| { vars.insert(s.to_string()); });

                // reverse so that application uses variable at the end of the list
                args.reverse();

                let body = body.translate(vars, funcs);

                args.iter()
                    .for_each(|s| { vars.remove(*s); });

                if args.len() == 0 {
                    body
                } else {
                    Term::Return(Box::new(
                        Term::Thunk(Box::new(
                            Term::Lambda { args, body: Box::new(body) }
                        ))
                    ))
                }
            },
            Decl::Stm(stm) => stm.translate(vars, funcs),
            _ => unreachable!()
        }
    }
}