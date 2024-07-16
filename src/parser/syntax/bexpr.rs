use super::expr::Expr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BExpr<'a> {
    Eq(Box<Expr<'a>>, Box<Expr<'a>>),
    NEq(Box<Expr<'a>>, Box<Expr<'a>>),
    Not(Box<Expr<'a>>)
}