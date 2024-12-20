use super::{arg::Arg, bexpr::BExpr, stm::Stm};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Cons(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    BExpr(BExpr),
    List(Vec<Expr>),
    Lambda(Arg, Box<Stm>),
    Ident(String),
    Nat(usize),
    Bool(bool),
    Pair(Box<Stm>, Box<Stm>),
    Stm(Box<Stm>)
}
