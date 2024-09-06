use super::{bexpr::BExpr, stm::Stm};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    BExpr(BExpr),
    List(Vec<Expr>),
    Lambda(String, Box<Stm>),
    Ident(String),
    Nat(usize),
    Bool(bool),
    Fold,
    Stm(Box<Stm>)
}

