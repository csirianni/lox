use crate::environment::Environment;
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
    let mut environment = Environment::new_top_level();
    for statement in statements.into_iter() {
        execute(statement, &mut environment)?;
    }
    Ok(())
}

fn execute(stmt: Stmt, environment: &mut Environment) -> Result<()> {
    match stmt {
        Stmt::Expression { expression } => {
            // We disregard the value because it is unused. We could actually optimize this
            // entire term out.
            let _ = evaluate(expression, environment)?;
        }
        Stmt::Print { expression } => {
            let value = evaluate(expression, environment)?;
            println!("{}", value);
        }
        Stmt::Var { name, initializer } => {
            let value = if initializer
                != (Expr::Literal {
                    value: Literal::None,
                }) {
                evaluate(initializer, environment)?
            } else {
                // An initializer is optional. The default value is None.
                Value::None
            };
            environment.define(&name.lexeme, value);
        }
    }
    Ok(())
}

fn evaluate(expression: Expr, environment: &mut Environment) -> Result<Value> {
    match expression {
        Expr::Literal { value } => match value {
            Literal::String(str) => Ok(Value::String(str)),
            Literal::Number(num) => Ok(Value::Number(num)),
            Literal::Boolean(bool) => Ok(Value::Boolean(bool)),
            Literal::None => Ok(Value::None),
        },
        Expr::Grouping { expression } => evaluate(*expression, environment),
        Expr::Unary { operator, right } => {
            let value = evaluate(*right, environment);
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
                let left = evaluate(*left, environment);
                let right = evaluate(*right, environment);
                Ok(Value::Boolean(left == right))
            } else {
                let Value::Number(left) = evaluate(*left, environment)? else {
                    return Err(RuntimeError {
                        token: operator,
                        message: "Expected type Value::Number for numeric operator".to_string(),
                    });
                };
                let Value::Number(right) = evaluate(*right, environment)? else {
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
        Expr::Variable { name } => match environment.get(&name) {
            Some(value) => Ok(value.clone()),
            None => Err(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable '{}'", name.lexeme),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_unary() {
        assert_eq!(
            evaluate(
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Bang,
                        lexeme: "".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    })
                },
                &mut Environment::new_top_level()
            ),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            evaluate(
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Bang,
                        lexeme: "".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    })
                },
                &mut Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );
        assert_eq!(
            evaluate(
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Minus,
                        lexeme: "".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(-13_f64)
                    })
                },
                &mut Environment::new_top_level(),
            ),
            Ok(Value::Number(13_f64))
        );
        assert_eq!(
            evaluate(
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Minus,
                        lexeme: "".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(13_f64)
                    })
                },
                &mut Environment::new_top_level(),
            ),
            Ok(Value::Number(-13_f64))
        );
    }

    #[test]
    fn test_evaluate_unary_bang_number() {
        assert_eq!(
            evaluate(
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Bang,
                        lexeme: "".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(0_f64),
                    })
                },
                &mut Environment::new_top_level(),
            ),
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
    fn test_evaluate_binary() {
        assert_eq!(
            evaluate(
                Expr::Binary {
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
                    })
                },
                &mut Environment::new_top_level(),
            ),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            evaluate(
                Expr::Binary {
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
                    })
                },
                &mut Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );
    }

    #[test]
    fn test_global_variable() {
        let mut environment = Environment::new_top_level();

        let definition = Stmt::Var {
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                literal: Literal::None,
                line: 0,
            },
            initializer: Expr::Literal {
                value: Literal::Number(5_f64),
            },
        };
        assert!(execute(definition, &mut environment).is_ok());

        let lookup = Expr::Variable {
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                literal: Literal::None,
                line: 0,
            },
        };
        let value = evaluate(lookup, &mut environment);
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), Value::Number(5_f64));
    }
}
