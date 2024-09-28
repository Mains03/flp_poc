use crate::{cbpv::{term_ptr::TermPtr, Term}, parser::syntax::{arg::Arg, decl::Decl, stm::Stm}};

use super::Translate;

impl Translate for Decl {
    fn translate(self) -> Term {
        match self {
            Decl::Func { name: _, args, body } => translate_func(args, body),
            _ => unreachable!()
        }
    }
}

fn translate_func(mut args: Vec<Arg>, body: Stm) -> Term {
    args.reverse();

    if args.len() > 0 {
        let arg = args.remove(args.len()-1);
        let body = translate_func_helper(args, body);

        Term::Thunk(TermPtr::from_term(Term::Lambda {
            arg, body: TermPtr::from_term(body)
        }))
    } else {
        translate_func_helper(args, body)
    }
}

fn translate_func_helper(mut args: Vec<Arg>, body: Stm) -> Term {
    if args.len() == 0 {
        body.translate()
    } else {
        let arg = args.remove(args.len()-1);
        let body = translate_func_helper(args, body);

        Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(Term::Lambda {
            arg, body: TermPtr::from_term(body)
        }))))
    }
}
