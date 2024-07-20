use type_error::TypeError;

use crate::parser::syntax::decl::*;

pub mod type_error;

pub fn check_type(ast: Prog) -> Result<(), TypeError> {
    Ok(())
}