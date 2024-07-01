use super::stm::Stms;

#[derive(Clone, Debug)]
pub enum Expr<'a> {
    App(Box<Expr<'a>>, Box<Expr<'a>>),
    Ident(&'a str),
    Nat(i64),
    Stms(Stms<'a>)
}