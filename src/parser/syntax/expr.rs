use super::stm::Stms;

pub enum Expr<'a> {
    App(Box<Expr<'a>>, Box<Expr<'a>>),
    Ident(&'a str),
    Nat(i64),
    Stms(Stms<'a>)
}