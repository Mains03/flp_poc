use term::Term;

use crate::parser::syntax::program::Decl;

pub mod term;
mod translate;
mod eval;

pub fn translate<'a>(decl: &'a Decl) -> Term<'a> {
    translate::translate(decl)
}