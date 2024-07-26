use crate::{cbpv::Term, parser::syntax::{decl::Decl, stm::Stm}};

use super::Translate;

impl Translate for Decl {
    fn translate(self) -> Term {
        match self {
            Decl::Func { name: _, args, body } => translate_func(args, body),
            _ => unreachable!()
        }
    }
}

fn translate_func(mut args: Vec<String>, body: Stm) -> Term {
    args.reverse();

    if args.len() > 0 {
        Term::Thunk(Box::new(Term::Lambda {
            var: args.remove(args.len()-1),
            body: Box::new(translate_func_helper(args, body))
        }))
    } else {
        translate_func_helper(args, body)
    }
}

fn translate_func_helper(mut args: Vec<String>, body: Stm) -> Term {
    if args.len() == 0 {
        body.translate()
    } else {
        Term::Return(Box::new(Term::Thunk(Box::new(Term::Lambda {
            var: args.remove(args.len()-1),
            body: Box::new(translate_func_helper(args, body))
        }))))
    }
}