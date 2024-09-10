use crate::{cbpv::{term_ptr::TermPtr, Term}, parser::syntax::{decl::Decl, stm::Stm}};

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
        let var = args.remove(args.len()-1);
        let body = translate_func_helper(args, body);

        let mut free_vars = body.free_vars();
        free_vars.remove(&var);

        Term::Thunk(TermPtr::from_term(Term::Lambda {
            var,
            free_vars,
            body: TermPtr::from_term(body)
        }))
    } else {
        translate_func_helper(args, body)
    }
}

fn translate_func_helper(mut args: Vec<String>, body: Stm) -> Term {
    if args.len() == 0 {
        body.translate()
    } else {
        let var = args.remove(args.len()-1);
        let body = translate_func_helper(args, body);

        let mut free_vars = body.free_vars();
        free_vars.remove(&var);

        Term::Return(TermPtr::from_term(Term::Thunk(TermPtr::from_term(Term::Lambda {
            var,
            free_vars,
            body: TermPtr::from_term(body)
        }))))
    }
}
