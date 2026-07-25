use crate::expr::Expr;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
}
