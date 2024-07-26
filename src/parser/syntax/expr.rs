use super::{bexpr::BExpr, stm::Stm};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    BExpr(BExpr),
    Ident(String),
    Nat(usize),
    Bool(bool),
    Stm(Box<Stm>)
}

