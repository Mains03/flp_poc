use super::{r#type::Type, stm::*};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Decl {
    FuncType {
        name: String,
        r#type: Type
    },
    Func {
        name: String,
        args: Vec<String>,
        body: Stm
    },
    Stm(Stm)
}