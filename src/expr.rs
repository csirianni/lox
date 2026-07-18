use std::fmt;

use crate::token::{Literal, Token};

#[derive(Debug, PartialEq)]
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
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal { value } => write!(f, "{:?}", value),
            Expr::Unary { operator, right } => write!(f, "({} {})", operator.lexeme, right),
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", operator.lexeme, left, right),
            Expr::Grouping { expression } => write!(f, "( {} )", expression),
        }
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
            "None"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Literal {
                    value: Literal::Boolean(true)
                }
            ),
            "Boolean(true)"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Literal {
                    value: Literal::Boolean(false)
                }
            ),
            "Boolean(false)"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Literal {
                    value: Literal::String("hi".to_string())
                }
            ),
            "String(\"hi\")"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Literal {
                    value: Literal::Number(42_f64)
                }
            ),
            "Number(42.0)"
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
                        literal: Literal::None,
                        line: 0
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    })
                }
            ),
            "(! Boolean(true))"
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
                        literal: Literal::None,
                        line: 0
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(2_f64)
                    })
                }
            ),
            "(+ Number(1.0) Number(2.0))"
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
            "( Number(1.0) )"
        );
    }
}
