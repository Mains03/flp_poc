use super::{arg::Arg, stm::*, r#type::Type};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Decl {
    FuncType {
        name: String,
        r#type: Type
    },
    Func {
        name: String,
        args: Vec<Arg>,
        body: Stm
    },
    Stm(Stm)
}