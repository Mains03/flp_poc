use crate::cbpv::Term;

use super::closure::Closure;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateTerm {
    Term(Term),
    Closure(Closure)
}