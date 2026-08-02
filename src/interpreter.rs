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
        // We print the value of expressions, even if print is not used. This is expected in
        // the REPL, but maybe not when running a file.
        Stmt::Expression { expression } | Stmt::Print { expression } => {
            let value = evaluate(expression, environment)?;
            println!("{}", value);
        }
        Stmt::If {
            keyword,
            condition,
            consq,
            altern,
        } => {
            if let Value::Boolean(b) = evaluate(condition, environment)? {
                if b {
                    execute(*consq, environment)?
                } else if altern.is_some() {
                    execute(*altern.unwrap(), environment)?
                }
            } else {
                return Err(RuntimeError {
                    token: keyword,
                    message: "Expect 'if' conditional to evaluate to a boolean".to_string(),
                });
            }
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
        Stmt::Block { statements } => {
            // TODO: We don't need to copy the environment here because it is read-only. How can we
            // implement that? Maybe Rc?
            let mut block = Environment::new_block(environment.clone());

            for stmt in statements {
                execute(stmt, &mut block)?;
            }
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
                        if right == 0_f64 {
                            Err(RuntimeError {
                                token: operator,
                                message: "Cannot divide by zero".to_string(),
                            })
                        } else {
                            Ok(Value::Number(left / right))
                        }
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
        Expr::Assign { name, value } => {
            let value = evaluate(*value, environment)?;

            match environment.assign(&name, value.clone()) {
                Some(_) => Ok(value),
                None => Err(RuntimeError {
                    token: name.clone(),
                    message: format!("Undefined variable '{}'", name.lexeme),
                }),
            }
        }
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
    fn test_divide_by_zero() {
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64)
                    }),
                    operator: Token {
                        token_type: TokenType::Slash,
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
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::Slash,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                message: "Cannot divide by zero".to_string()
            })
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

    #[test]
    fn test_assignment() {
        let mut environment = Environment::new_top_level();

        let assignment = Expr::Assign {
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                literal: Literal::None,
                line: 0,
            },
            value: Box::new(Expr::Literal {
                value: Literal::Number(4_f64),
            }),
        };
        // We start at evaluate() here because assignment is an expression, not a statement.
        assert_eq!(
            evaluate(assignment.clone(), &mut environment),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "a".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                message: "Undefined variable 'a'".to_string()
            })
        );

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

        assert_eq!(
            evaluate(assignment, &mut environment),
            Ok(Value::Number(4_f64))
        );
    }

    #[test]
    fn test_valid_if_statement() {
        let mut environment = Environment::new_top_level();

        let if_statement = Stmt::If {
            keyword: Token {
                token_type: TokenType::If,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            condition: Expr::Literal {
                value: Literal::Boolean(true),
            },
            consq: Box::new(Stmt::Expression {
                expression: Expr::Literal {
                    value: Literal::Boolean(false),
                },
            }),
            altern: None,
        };

        assert_eq!(execute(if_statement, &mut environment), Ok(()));
    }

    #[test]
    fn test_invalid_if_statement() {
        let mut environment = Environment::new_top_level();

        let if_statement = Stmt::If {
            keyword: Token {
                token_type: TokenType::If,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            condition: Expr::Literal {
                value: Literal::Number(1_f64),
            },
            consq: Box::new(Stmt::Expression {
                expression: Expr::Literal {
                    value: Literal::Boolean(false),
                },
            }),
            altern: None,
        };

        assert_eq!(
            execute(if_statement, &mut environment),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::If,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                message: "Expect 'if' conditional to evaluate to a boolean".to_string()
            })
        );
    }

    #[test]
    fn test_if_statement_short_circuiting() {
        let mut environment = Environment::new_top_level();

        let if_statement_consq = Stmt::If {
            keyword: Token {
                token_type: TokenType::If,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            condition: Expr::Literal {
                value: Literal::Boolean(true),
            },
            consq: Box::new(Stmt::Expression {
                expression: Expr::Literal {
                    value: Literal::None,
                },
            }),
            // 5 / 0 should not be evaluated; otherwise, it's a runtime error.
            altern: Some(Box::new(Stmt::Expression {
                expression: Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64),
                    }),
                    operator: Token {
                        token_type: TokenType::Slash,
                        lexeme: "".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(0_f64),
                    }),
                },
            })),
        };
        assert_eq!(execute(if_statement_consq, &mut environment), Ok(()));

        let if_statement_altern = Stmt::If {
            keyword: Token {
                token_type: TokenType::If,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            condition: Expr::Literal {
                value: Literal::Boolean(false),
            },
            // 5 / 0 should not be evaluated; otherwise, it's a runtime error.
            consq: Box::new(Stmt::Expression {
                expression: Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64),
                    }),
                    operator: Token {
                        token_type: TokenType::Slash,
                        lexeme: "".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(0_f64),
                    }),
                },
            }),
            altern: None,
        };
        assert_eq!(execute(if_statement_altern, &mut environment), Ok(()));
    }
}
