use crate::{expr::Expr, token::Token};

/// statement      → exprStmt
///                | ifStmt
///                | printStmt
///                | varStmt
///                | block
///                | whileStmt;
///
/// block          → "{" declaration* "}" ;
/// ifStmt         → "if" "(" expression ")" statement
///                ( "else" statement )? ;
/// whileStmt      → "while" "(" expression ")" statement ;

#[derive(Debug, PartialEq, Clone)]
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
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
