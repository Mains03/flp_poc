use super::expr::Expr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BExpr {
    Eq(Box<Expr>, Box<Expr>),
    NEq(Box<Expr>, Box<Expr>),
    Not(Box<Expr>)
}