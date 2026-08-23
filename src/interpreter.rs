use std::cell::RefCell;
use std::rc::Rc;

use crate::environment::Environment;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token};
use crate::token_type::TokenType;
use crate::value::Value;

type Result<T> = std::result::Result<T, RuntimeError>;

type ExecuteResult<T> = std::result::Result<T, ExecuteError>;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum ExecuteError {
    RuntimeError(RuntimeError),
    // We use an error enum variant to allow execute() to propogate return values up the call stack
    // using the ? operator.
    ControlFlow(Value),
}

impl From<RuntimeError> for ExecuteError {
    fn from(err: RuntimeError) -> Self {
        ExecuteError::RuntimeError(err)
    }
}

pub fn interpret(statements: Vec<Stmt>) -> Result<()> {
    let environment = Environment::new_top_level();
    for statement in statements.into_iter() {
        match execute(statement, environment.clone()) {
            // Swallow return values because they don't need to be handled by the caller.
            Ok(()) | Err(ExecuteError::ControlFlow(_)) => {}
            Err(ExecuteError::RuntimeError(e)) => return Err(e),
        }
    }

    Ok(())
}

// TODO: Consider returning Result<Value> here instead of the unit type. Value::None could be
// considered the unit type of our language, or we could add a dedicated value to statements that
// evaluate to nothing.
fn execute(stmt: Stmt, environment: Rc<RefCell<Environment>>) -> ExecuteResult<()> {
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
        Stmt::If {
            keyword,
            condition,
            consq,
            altern,
        } => {
            if let Value::Boolean(b) = evaluate(condition, environment.clone())? {
                if b {
                    execute(*consq, environment)?
                } else if altern.is_some() {
                    execute(*altern.unwrap(), environment.clone())?
                }
            } else {
                return Err(ExecuteError::RuntimeError(RuntimeError {
                    token: keyword,
                    message: "Expect 'if' conditional to evaluate to a boolean".to_string(),
                }));
            }
        }
        Stmt::Var { name, initializer } => {
            let value = if initializer
                != (Expr::Literal {
                    value: Literal::None,
                }) {
                evaluate(initializer, environment.clone())?
            } else {
                // An initializer is optional. The default value is None.
                Value::None
            };
            environment.borrow_mut().define(&name.lexeme, value);
        }
        Stmt::Fun { name, params, body } => {
            environment.borrow_mut().define(
                &name.lexeme,
                Value::Fun {
                    params,
                    body,
                    // FIX: This is dynamic scope because it is a shared reference.
                    environment: environment.clone(),
                },
            );
        }
        Stmt::Return { keyword: _, value } => {
            let return_value = match value {
                Some(expr) => evaluate(expr, environment)?,
                None => Value::None,
            };
            return Err(ExecuteError::ControlFlow(return_value));
        }
        Stmt::Block { statements } => {
            let block = Environment::new_block(environment);

            for stmt in statements {
                execute(stmt, block.clone())?;
            }
        }
        Stmt::While { condition, body } => {
            // TODO: Consider passing by reference to avoid copies. interpret() or above would own
            // these structs.
            while is_true(evaluate(condition.clone(), environment.clone()))? {
                execute(*body.clone(), environment.clone())?
            }
        }
    }
    Ok(())
}

fn evaluate(expression: Expr, environment: Rc<RefCell<Environment>>) -> Result<Value> {
    match expression {
        Expr::Literal { value } => match value {
            Literal::String(str) => Ok(Value::String(str)),
            Literal::Number(num) => Ok(Value::Number(num)),
            Literal::Boolean(bool) => Ok(Value::Boolean(bool)),
            Literal::None => Ok(Value::None),
        },
        Expr::Grouping { expression } => evaluate(*expression, environment.clone()),
        Expr::Unary { operator, right } => {
            let value = evaluate(*right, environment.clone());
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
        } => match operator.token_type {
            TokenType::BangEqual | TokenType::EqualEqual => {
                let left = evaluate(*left, environment.clone());
                let right = evaluate(*right, environment.clone());
                // FIX: This is not correct because we are comparing results, meaning that we do not
                // use the typical bubble up approach if evaluate() returns an error.
                Ok(Value::Boolean(left == right))
            }
            TokenType::Or => {
                let Value::Boolean(left) = evaluate(*left, environment.clone())? else {
                    return Err(RuntimeError {
                        token: operator,
                        message: "Expected type Value::Boolean for OR operator".to_string(),
                    });
                };
                if !left {
                    let Value::Boolean(right) = evaluate(*right, environment.clone())? else {
                        return Err(RuntimeError {
                            token: operator,
                            message: "Expected type Value::Boolean for OR operator".to_string(),
                        });
                    };
                    Ok(Value::Boolean(left || right))
                } else {
                    // Short-circuit when left is true.
                    Ok(Value::Boolean(left))
                }
            }
            TokenType::And => {
                let Value::Boolean(left) = evaluate(*left, environment.clone())? else {
                    return Err(RuntimeError {
                        token: operator,
                        message: "Expected type Value::Boolean for AND operator".to_string(),
                    });
                };
                if left {
                    let Value::Boolean(right) = evaluate(*right, environment.clone())? else {
                        return Err(RuntimeError {
                            token: operator,
                            message: "Expected type Value::Boolean for AND operator".to_string(),
                        });
                    };
                    Ok(Value::Boolean(left && right))
                } else {
                    // Short-circuit when left is false.
                    Ok(Value::Boolean(left))
                }
            }
            TokenType::Minus
            | TokenType::Plus
            | TokenType::Slash
            | TokenType::Star
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => {
                let Value::Number(left) = evaluate(*left, environment.clone())? else {
                    return Err(RuntimeError {
                        token: operator,
                        message: "Expected type Value::Number for numeric operator".to_string(),
                    });
                };
                let Value::Number(right) = evaluate(*right, environment.clone())? else {
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
            _ => panic!(
                "Unexpected binary operator token type: {:?}",
                operator.token_type
            ),
        },
        Expr::Variable { name } => match environment.borrow().get(&name) {
            Some(value) => Ok(value.clone()),
            None => Err(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable '{}'", name.lexeme),
            }),
        },
        Expr::Assign { name, value } => {
            let value = evaluate(*value, environment.clone())?;

            match environment.borrow_mut().assign(&name, value.clone()) {
                Some(_) => Ok(value),
                None => Err(RuntimeError {
                    token: name.clone(),
                    message: format!("Undefined variable '{}'", name.lexeme),
                }),
            }
        }
        Expr::Fun { params, body } => Ok(Value::Fun {
            params,
            body,
            environment: environment.clone(),
        }),
        Expr::Call {
            fun,
            paren,
            arguments,
        } => {
            if let Value::Fun {
                params,
                body,
                environment,
            } = evaluate(*fun.clone(), environment.clone())?
            {
                // TODO: Rename `environment` because the shadowing is confusing.
                if params.len() != arguments.len() {
                    return Err(RuntimeError {
                        token: paren,
                        message: format!(
                            "Expected {} function arguments but got {}",
                            params.len(),
                            arguments.len()
                        ),
                    });
                }
                for (param, argument) in std::iter::zip(params, arguments) {
                    environment
                        .borrow_mut()
                        .define(&param.lexeme, evaluate(argument, environment.clone())?);
                }

                for stmt in body {
                    match execute(stmt, environment.clone()) {
                        Ok(()) => {}
                        Err(ExecuteError::ControlFlow(value)) => return Ok(value),
                        Err(ExecuteError::RuntimeError(e)) => return Err(e),
                    }
                }
                Ok(Value::None)
            } else {
                Err(RuntimeError {
                    token: paren,
                    message: format!("{:?} is not a function", fun),
                })
            }
        }
    }
}

fn is_true(result: Result<Value>) -> ExecuteResult<bool> {
    if let Value::Boolean(bool) = result? {
        Ok(bool)
    } else {
        Err(ExecuteError::RuntimeError(RuntimeError {
            token: todo!(),
            message: "Expected type Value::Boolean for WHILE loop".to_string(),
        }))
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
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    })
                },
                Environment::new_top_level()
            ),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            evaluate(
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Bang,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );
        assert_eq!(
            evaluate(
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Minus,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(-13_f64)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Number(13_f64))
        );
        assert_eq!(
            evaluate(
                Expr::Unary {
                    operator: Token {
                        token_type: TokenType::Minus,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(13_f64)
                    })
                },
                Environment::new_top_level(),
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
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(0_f64),
                    })
                },
                Environment::new_top_level(),
            ),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::Bang,
                    ..Default::default()
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
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::None
                    })
                },
                Environment::new_top_level(),
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
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(0_f64)
                    })
                },
                Environment::new_top_level(),
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
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(0_f64)
                    })
                },
                Environment::new_top_level(),
            ),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::Slash,
                    ..Default::default()
                },
                message: "Cannot divide by zero".to_string()
            })
        );
    }

    #[test]
    fn test_global_variable() {
        let environment = Environment::new_top_level();

        let definition = Stmt::Var {
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                ..Default::default()
            },
            initializer: Expr::Literal {
                value: Literal::Number(5_f64),
            },
        };
        assert!(execute(definition, environment.clone()).is_ok());

        let lookup = Expr::Variable {
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                ..Default::default()
            },
        };
        let value = evaluate(lookup, environment.clone());
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), Value::Number(5_f64));
    }

    #[test]
    fn test_assignment() {
        let environment = Environment::new_top_level();

        let assignment = Expr::Assign {
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                ..Default::default()
            },
            value: Box::new(Expr::Literal {
                value: Literal::Number(4_f64),
            }),
        };
        // We start at evaluate() here because assignment is an expression, not a statement.
        assert_eq!(
            evaluate(assignment.clone(), environment.clone()),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "a".to_string(),
                    ..Default::default()
                },
                message: "Undefined variable 'a'".to_string()
            })
        );

        let definition = Stmt::Var {
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                ..Default::default()
            },
            initializer: Expr::Literal {
                value: Literal::Number(5_f64),
            },
        };
        assert!(execute(definition, environment.clone()).is_ok());

        assert_eq!(
            evaluate(assignment, environment.clone()),
            Ok(Value::Number(4_f64))
        );
    }

    #[test]
    fn test_valid_if_statement() {
        let environment = Environment::new_top_level();

        let if_statement = Stmt::If {
            keyword: Token {
                token_type: TokenType::If,
                ..Default::default()
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

        assert_eq!(execute(if_statement, environment.clone()), Ok(()));
    }

    #[test]
    fn test_invalid_if_statement() {
        let environment = Environment::new_top_level();

        let if_statement = Stmt::If {
            keyword: Token {
                token_type: TokenType::If,
                ..Default::default()
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
            execute(if_statement, environment),
            Err(ExecuteError::RuntimeError(RuntimeError {
                token: Token {
                    token_type: TokenType::If,
                    ..Default::default()
                },
                message: "Expect 'if' conditional to evaluate to a boolean".to_string()
            }))
        );
    }

    #[test]
    fn test_if_statement_short_circuiting() {
        let environment = Environment::new_top_level();

        let if_statement_consq = Stmt::If {
            keyword: Token {
                token_type: TokenType::If,
                ..Default::default()
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
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(0_f64),
                    }),
                },
            })),
        };
        assert_eq!(execute(if_statement_consq, environment.clone()), Ok(()));

        let if_statement_altern = Stmt::If {
            keyword: Token {
                token_type: TokenType::If,
                ..Default::default()
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
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(0_f64),
                    }),
                },
            }),
            altern: None,
        };
        assert_eq!(execute(if_statement_altern, environment.clone()), Ok(()));
    }

    #[test]
    fn test_logical_or_true_table() {
        // True || False == True
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    }),
                    operator: Token {
                        token_type: TokenType::Or,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(true))
        );

        // False || True == True
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    }),
                    operator: Token {
                        token_type: TokenType::Or,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(true))
        );

        // True || True == True
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    }),
                    operator: Token {
                        token_type: TokenType::Or,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(true))
        );

        // False || False == False
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    }),
                    operator: Token {
                        token_type: TokenType::Or,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );
    }

    #[test]
    fn test_logical_or_runtime_type_error() {
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    }),
                    operator: Token {
                        token_type: TokenType::Or,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64)
                    })
                },
                Environment::new_top_level(),
            ),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::Or,
                    ..Default::default()
                },
                message: "Expected type Value::Boolean for OR operator".to_string()
            })
        );
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64)
                    }),
                    operator: Token {
                        token_type: TokenType::Or,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    })
                },
                Environment::new_top_level(),
            ),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::Or,
                    ..Default::default()
                },
                message: "Expected type Value::Boolean for OR operator".to_string()
            })
        );
        // Short-circuiting avoids runtime type error.
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    }),
                    operator: Token {
                        token_type: TokenType::Or,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn test_logical_and_true_table() {
        // True || False == False
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    }),
                    operator: Token {
                        token_type: TokenType::And,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );

        // False || True == False
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    }),
                    operator: Token {
                        token_type: TokenType::And,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );

        // True || True == True
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    }),
                    operator: Token {
                        token_type: TokenType::And,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(true))
        );

        // False || False == False
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    }),
                    operator: Token {
                        token_type: TokenType::And,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );
    }

    #[test]
    fn test_logical_and_runtime_type_error() {
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(true)
                    }),
                    operator: Token {
                        token_type: TokenType::And,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64)
                    })
                },
                Environment::new_top_level(),
            ),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::And,
                    ..Default::default()
                },
                message: "Expected type Value::Boolean for AND operator".to_string()
            })
        );
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64)
                    }),
                    operator: Token {
                        token_type: TokenType::And,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    })
                },
                Environment::new_top_level(),
            ),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::And,
                    ..Default::default()
                },
                message: "Expected type Value::Boolean for AND operator".to_string()
            })
        );
        // Short-circuiting avoids runtime type error.
        assert_eq!(
            evaluate(
                Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Boolean(false)
                    }),
                    operator: Token {
                        token_type: TokenType::And,
                        ..Default::default()
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64)
                    })
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );
    }

    #[test]
    #[should_panic(expected = "Unexpected binary operator token type: Print")]
    fn test_unexpected_binary_operator() {
        evaluate(
            Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(5_f64),
                }),
                operator: Token {
                    token_type: TokenType::Print,
                    ..Default::default()
                },
                right: Box::new(Expr::Literal {
                    value: Literal::Boolean(false),
                }),
            },
            Environment::new_top_level(),
        )
        .unwrap();
    }

    #[test]
    fn test_function_application() {
        // Zero args.
        assert_eq!(
            evaluate(
                Expr::Call {
                    fun: Box::new(Expr::Fun {
                        params: Vec::new(),
                        body: vec![Stmt::Return {
                            keyword: Token {
                                token_type: TokenType::Return,
                                ..Default::default()
                            },
                            value: Some(Expr::Literal {
                                value: Literal::Number(5_f64)
                            }),
                        }],
                    }),
                    paren: Token {
                        token_type: TokenType::LeftParen,
                        ..Default::default()
                    },
                    arguments: Vec::new(),
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Number(5_f64))
        );

        // One arg.
        assert_eq!(
            evaluate(
                Expr::Call {
                    fun: Box::new(Expr::Fun {
                        params: vec![Token {
                            token_type: TokenType::Identifier,
                            lexeme: "foo".to_string(),
                            ..Default::default()
                        }],
                        body: vec![Stmt::Return {
                            keyword: Token {
                                token_type: TokenType::Return,
                                ..Default::default()
                            },
                            value: Some(Expr::Variable {
                                name: Token {
                                    token_type: TokenType::Identifier,
                                    lexeme: "foo".to_string(),
                                    ..Default::default()
                                }
                            }),
                        }],
                    }),
                    paren: Token {
                        token_type: TokenType::LeftParen,
                        ..Default::default()
                    },
                    arguments: vec![Expr::Literal {
                        value: Literal::Boolean(false),
                    }],
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Boolean(false))
        );

        // Two args.
        assert_eq!(
            evaluate(
                Expr::Call {
                    fun: Box::new(Expr::Fun {
                        params: vec![
                            Token {
                                token_type: TokenType::Identifier,
                                lexeme: "x".to_string(),
                                ..Default::default()
                            },
                            Token {
                                token_type: TokenType::Identifier,
                                lexeme: "y".to_string(),
                                ..Default::default()
                            }
                        ],
                        body: vec![Stmt::Return {
                            keyword: Token {
                                token_type: TokenType::Return,
                                ..Default::default()
                            },
                            value: Some(Expr::Binary {
                                left: Box::new(Expr::Variable {
                                    name: Token {
                                        token_type: TokenType::Identifier,
                                        lexeme: "x".to_string(),
                                        ..Default::default()
                                    }
                                }),
                                operator: Token {
                                    token_type: TokenType::Plus,
                                    ..Default::default()
                                },
                                right: Box::new(Expr::Variable {
                                    name: Token {
                                        token_type: TokenType::Identifier,
                                        lexeme: "y".to_string(),
                                        ..Default::default()
                                    }
                                }),
                            }),
                        }]
                    }),
                    paren: Token {
                        token_type: TokenType::LeftParen,
                        ..Default::default()
                    },
                    arguments: vec![
                        Expr::Literal {
                            value: Literal::Number(1_f64),
                        },
                        Expr::Literal {
                            value: Literal::Number(2_f64),
                        }
                    ],
                },
                Environment::new_top_level(),
            ),
            Ok(Value::Number(3_f64))
        );

        assert_eq!(
            evaluate(
                Expr::Call {
                    fun: Box::new(Expr::Fun {
                        params: vec![
                            Token {
                                token_type: TokenType::Identifier,
                                lexeme: "x".to_string(),
                                ..Default::default()
                            },
                            Token {
                                token_type: TokenType::Identifier,
                                lexeme: "y".to_string(),
                                ..Default::default()
                            }
                        ],
                        body: vec![Stmt::Expression {
                            expression: Expr::Binary {
                                left: Box::new(Expr::Variable {
                                    name: Token {
                                        token_type: TokenType::Identifier,
                                        lexeme: "x".to_string(),
                                        ..Default::default()
                                    }
                                }),
                                operator: Token {
                                    token_type: TokenType::Plus,
                                    ..Default::default()
                                },
                                right: Box::new(Expr::Variable {
                                    name: Token {
                                        token_type: TokenType::Identifier,
                                        lexeme: "y".to_string(),
                                        ..Default::default()
                                    }
                                }),
                            }
                        }]
                    }),
                    paren: Token {
                        token_type: TokenType::LeftParen,
                        ..Default::default()
                    },
                    arguments: vec![Expr::Literal {
                        value: Literal::Boolean(false),
                    }],
                },
                Environment::new_top_level(),
            ),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::LeftParen,
                    ..Default::default()
                },
                message: "Expected 2 function arguments but got 1".to_string()
            })
        );

        // `fun` is not a function.
        assert_eq!(
            evaluate(
                Expr::Call {
                    fun: Box::new(Expr::Literal {
                        value: Literal::Number(5_f64)
                    }),
                    paren: Token {
                        token_type: TokenType::LeftParen,
                        ..Default::default()
                    },
                    arguments: Vec::new(),
                },
                Environment::new_top_level(),
            ),
            Err(RuntimeError {
                token: Token {
                    token_type: TokenType::LeftParen,
                    ..Default::default()
                },
                message: "Literal { value: Number(5.0) } is not a function".to_string()
            })
        );

        // TODO: Test non-empty TLE.
    }
}
