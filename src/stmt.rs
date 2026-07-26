use crate::{expr::Expr, token::Token};

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: Token, initializer: Expr },
}
