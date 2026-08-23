use std::fmt;

use crate::token::{Literal, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Fun {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Call {
        fun: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expr(").unwrap();
        match self {
            Expr::Literal { value } => {
                write!(f, "{:?}", value).unwrap();
            }
            Expr::Unary { operator, right } => {
                write!(f, "{} {}", operator.lexeme, right).unwrap();
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "{} {} {}", operator.lexeme, left, right).unwrap();
            }
            Expr::Grouping { expression } => {
                write!(f, "Grouping({})", expression).unwrap();
            }
            Expr::Variable { name } => {
                write!(f, "{:?}", name).unwrap();
            }
            Expr::Assign { name, value } => {
                write!(f, "{:?} {}", name, value).unwrap();
            }
            Expr::Fun { params, body } => {
                write!(f, "Fun(params: {:?}, body: {})", params, body).unwrap();
            }
            Expr::Call {
                fun: func,
                paren,
                arguments,
            } => {
                write!(
                    f,
                    "Call(func: {:?}, paren: {:?}, arguments: {:?})",
                    func, paren, arguments
                )
                .unwrap();
            }
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_type::TokenType;

    #[test]
    fn test_display_literal() {
        assert_eq!(
            format!(
                "{}",
                Expr::Literal {
                    value: Literal::None
                }
            ),
            "Expr(None)"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Literal {
                    value: Literal::Boolean(true)
                }
            ),
            "Expr(Boolean(true))"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Literal {
                    value: Literal::String("hi".to_string())
                }
            ),
            "Expr(String(\"hi\"))"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Literal {
                    value: Literal::Number(42_f64)
                }
            ),
            "Expr(Number(42.0))"
        );
    }

    #[test]
    fn test_display_unary() {
        assert_eq!(
            format!(
                "{}",
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Bang,
                        lexeme: "!".to_string(),
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    })
                }
            ),
            "Expr(! Expr(Boolean(true)))"
        );
    }

    #[test]
    fn test_display_binary() {
        assert_eq!(
            format!(
                "{}",
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(1_f64)
                    }),
                    operator: Token {
                        token_type: TokenType::Plus,
                        lexeme: "+".to_string(),
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(2_f64)
                    })
                }
            ),
            "Expr(+ Expr(Number(1.0)) Expr(Number(2.0)))"
        );
    }

    #[test]
    fn test_display_grouping() {
        assert_eq!(
            format!(
                "{}",
                Expr::Grouping {
                    expression: Box::new(Expr::Literal {
                        value: Literal::Number(1_f64)
                    }),
                }
            ),
            "Expr(Grouping(Expr(Number(1.0))))"
        );
    }

    #[test]
    fn test_display_fun() {
        assert_eq!(
            format!(
                "{}",
                Expr::Fun {
                    params: vec!["a".to_string(), "b".to_string()],
                    body: Box::new(Expr::Literal {
                        value: Literal::None
                    })
                }
            ),
            "Expr(Fun(params: [\"a\", \"b\"], body: Expr(None)))"
        );
    }
}
