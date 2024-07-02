use super::{r#type::Type, expr::Expr};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stm<'a> {
    If {
        cond: Box<Stm<'a>>,
        then: Box<Stm<'a>>,
        r#else: Box<Stm<'a>>
    },
    Let {
        var: &'a str,
        val: Box<Stm<'a>>,
        body: Box<Stm<'a>>
    },
    Exists {
        var: &'a str,
        r#type: Type<'a>,
        body: Box<Stm<'a>>
    },
    Equate {
        lhs: Expr<'a>,
        rhs: Expr<'a>,
        body: Box<Stm<'a>>
    },
    Choice(Vec<Expr<'a>>),
    Expr(Expr<'a>)
}