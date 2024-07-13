use super::stm::Stm;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr<'a> {
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    App(Box<Expr<'a>>, Box<Expr<'a>>),
    Ident(&'a str),
    Nat(usize),
    Stm(Box<Stm<'a>>)
}