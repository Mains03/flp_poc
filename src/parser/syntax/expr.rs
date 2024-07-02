use super::stm::Stm;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr<'a> {
    App(Box<Expr<'a>>, Box<Expr<'a>>),
    Ident(&'a str),
    Nat(i64),
    Stm(Box<Stm<'a>>)
}