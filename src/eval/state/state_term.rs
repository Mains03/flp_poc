use crate::cbpv::Term;

use super::closure::Closure;

#[derive(Clone, Debug)]
pub enum StateTerm {
    Term(Term),
    Closure(Closure)
}