use super::{bexpr::BExpr, stm::Stm};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr<'a> {
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    App(Box<Expr<'a>>, Box<Expr<'a>>),
    BExpr(BExpr<'a>),
    Ident(&'a str),
    Nat(usize),
    Bool(bool),
    Stm(Box<Stm<'a>>)
}