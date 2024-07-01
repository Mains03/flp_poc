use super::{r#type::Type, arg::Arg, stm::*};

pub type Prog<'a> = Vec<Decl<'a>>;

#[derive(Debug)]
pub enum Decl<'a> {
    FuncType {
        name: &'a str,
        r#type: Type<'a>
    },
    Func {
        name: &'a str,
        args: Vec<Arg<'a>>,
        body: Stms<'a>
    },
    Stms(Stms<'a>)
}