use crate::{eval::state::state_term::StateTerm, parser::syntax::r#type::Type};

#[derive(Clone, Debug)]
pub enum EnvValue {
    Term(StateTerm),
    Type(Type),
}