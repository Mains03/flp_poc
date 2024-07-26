use type_error::TypeError;

use crate::parser::syntax::decl::Decl;

pub mod type_error;

pub fn check_type(ast: Vec<Decl>) -> Result<(), TypeError> {
    Ok(())
}