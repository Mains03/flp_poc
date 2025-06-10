use super::{arg::Arg, bexpr::BExpr, stm::Stm};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Zero,
    Succ(Box<Expr>),
    Nil,
    Cons(Box<Expr>, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    BExpr(BExpr),
    List(Vec<Expr>),
    Lambda(Arg, Box<Stm>),
    Ident(String),
    Nat(usize),
    Bool(bool),
    Pair(Box<Expr>, Box<Expr>),
    Stm(Box<Stm>)
}

impl Expr {
    pub fn strip_parentheses(self) -> Expr {
        let mut e = self;
        loop {
            match e {
                Expr::Stm(stm) => match *stm {
                    Stm::Expr(expr) => e = expr,
                    stm => e = Expr::Stm(Box::new(stm))
                },
                _ => break
            }
        }

        e
    }
}