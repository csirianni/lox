use crate::{expr::Expr, token::Token};

/// statement      → exprStmt
///                | ifStmt
///                | printStmt
///                | varStmt
///                | block ;
///
/// block          → "{" declaration* "}" ;
/// ifStmt         → "if" "(" expression ")" statement
///                ( "else" statement )? ;
#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },
    If {
        /// The `if` keyword token is used to report a `RuntimeError` if `condition` does not
        /// evaluate to a boolean.
        keyword: Token,
        condition: Expr,
        consq: Box<Stmt>,
        altern: Option<Box<Stmt>>,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Expr,
    },
    Block {
        statements: Vec<Stmt>,
    },
}
