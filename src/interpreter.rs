use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token};
use crate::token_type::TokenType;
use crate::value::Value;

type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

pub fn interpret(statements: Vec<Stmt>) -> Result<()> {
    for statement in statements.into_iter() {
        execute(statement)?;
    }
    Ok(())
}

fn execute(stmt: Stmt) -> Result<()> {
    match stmt {
        Stmt::Expression { expression } => {
            // We disregard the value because it is unused. We could actually optimize this
            // entire term out.
            let _ = evaluate(expression)?;
        }
        Stmt::Print { expression } => {
            let value = evaluate(expression)?;
            println!("{}", value);
        }
    }
    Ok(())
}

fn evaluate(expression: Expr) -> Result<Value> {
    match expression {
        Expr::Literal { value } => match value {
            Literal::String(str) => Ok(Value::String(str)),
            Literal::Number(num) => Ok(Value::Number(num)),
            Literal::Boolean(bool) => Ok(Value::Boolean(bool)),
            Literal::None => Ok(Value::None),
        },
        Expr::Grouping { expression } => evaluate(*expression),
        Expr::Unary { operator, right } => {
            let value = evaluate(*right);
            match operator.token_type {
                TokenType::Minus => {
                    let Ok(Value::Number(num)) = value else {
                        return Err(RuntimeError {
                            token: operator,
                            message: "Expected type Value::Number for - operator".to_string(),
                        });
                    };
                    Ok(Value::Number(-num))
                }
                TokenType::Bang => {
                    // No truthy/falsey values. All expressions must explicitly evaluate to a
                    // Value::Boolean. Otherwise, we produce a runtime type error.
                    let Ok(Value::Boolean(bool)) = value else {
                        return Err(RuntimeError {
                            token: operator,
                            message: "Expected type Value::Boolean for ! operator".to_string(),
                        });
                    };
                    if bool {
                        Ok(Value::Boolean(false))
                    } else {
                        Ok(Value::Boolean(true))
                    }
                }
                _ => unreachable!(),
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            if operator.token_type == TokenType::BangEqual
                || operator.token_type == TokenType::EqualEqual
            {
                let left = evaluate(*left);
                let right = evaluate(*right);
                Ok(Value::Boolean(left == right))
            } else {
                let Value::Number(left) = evaluate(*left)? else {
                    return Err(RuntimeError {
                        token: operator,
                        message: "Expected type Value::Number for numeric operator".to_string(),
                    });
                };
                let Value::Number(right) = evaluate(*right)? else {
                    return Err(RuntimeError {
                        token: operator,
                        message: "Expected type Value::Number for numeric operator".to_string(),
                    });
                };
                match operator.token_type {
                    TokenType::Minus => Ok(Value::Number(left - right)),
                    TokenType::Plus => Ok(Value::Number(left + right)),
                    TokenType::Slash => {
                        // TODO: We should probably handle division by zero as a runtime error.
                        Ok(Value::Number(left / right))
                    }
                    TokenType::Star => Ok(Value::Number(left * right)),
                    TokenType::Greater => Ok(Value::Boolean(left > right)),
                    TokenType::GreaterEqual => Ok(Value::Boolean(left >= right)),
                    TokenType::Less => Ok(Value::Boolean(left < right)),
                    TokenType::LessEqual => Ok(Value::Boolean(left <= right)),
                    _ => unreachable!(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_unary() {
        assert_eq!(
            evaluate(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Bang,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    value: Literal::Boolean(false)
                })
            }),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            evaluate(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Bang,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    value: Literal::Boolean(true)
                })
            }),
            Ok(Value::Boolean(false))
        );
        assert_eq!(
            evaluate(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Minus,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    value: Literal::Number(-13_f64)
                })
            }),
            Ok(Value::Number(13_f64))
        );
        assert_eq!(
            evaluate(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Minus,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    value: Literal::Number(13_f64)
                })
            }),
            Ok(Value::Number(-13_f64))
        );
    }

    #[test]
    fn test_interpret_unary_bang_number() {
        assert_eq!(
            evaluate(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Bang,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    value: Literal::Number(0_f64),
                }),
            }),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::Bang,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                message: "Expected type Value::Boolean for ! operator".to_string()
            })
        );
    }

    #[test]
    fn test_interpret_binary() {
        assert_eq!(
            evaluate(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::None
                }),
                operator: Token {
                    token_type: TokenType::EqualEqual,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    value: Literal::None
                }),
            }),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            evaluate(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::None
                }),
                operator: Token {
                    token_type: TokenType::EqualEqual,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    value: Literal::Number(0_f64)
                }),
            }),
            Ok(Value::Boolean(false))
        );
    }
}
