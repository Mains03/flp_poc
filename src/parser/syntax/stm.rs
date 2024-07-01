use super::{r#type::Type, expr::Expr};

pub type Stms<'a> = Vec<Stm<'a>>;

pub enum Stm<'a> {
    If {
        cond: Stms<'a>,
        then: Stms<'a>,
        r#else: Stms<'a>
    },
    Let {
        var: &'a str,
        val: Stms<'a>,
        body: Stms<'a>
    },
    Exists {
        var: &'a str,
        r#type: Type<'a>
    },
    Equate {
        lhs: Expr<'a>,
        rhs: Expr<'a>
    },
    Choice(Vec<Expr<'a>>)
}