use super::expr::Expr;

// Single instance of [pattern] -> [expr] in case statement
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseArm {
    pub pattern: Expr,
    pub expression: Expr
}