use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::{environment::Environment, expr::Expr};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Fun {
        params: Vec<String>,
        body: Box<Expr>,
        environment: Rc<RefCell<Environment>>,
    },
    None,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(str) => write!(f, "Value({:?})", str),
            Value::Number(num) => write!(f, "Value({:?})", num),
            Value::Boolean(bool) => write!(f, "Value({:?})", bool),
            Value::Fun {
                params,
                body,
                environment,
            } => {
                write!(
                    f,
                    "Value(Fun(params: {:?}, body: {}, environment: {:?}))",
                    params, body, environment
                )
            }
            Value::None => write!(f, "Value(none)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Literal;

    #[test]
    fn test_display_string() {
        assert_eq!(
            format!("{}", Value::String("hi".to_string())),
            "Value(\"hi\")"
        );
    }

    #[test]
    fn test_display_number() {
        assert_eq!(format!("{}", Value::Number(11_f64)), "Value(11.0)");
    }

    #[test]
    fn test_display_boolean() {
        assert_eq!(format!("{}", Value::Boolean(false)), "Value(false)");
    }

    #[test]
    fn test_display_fun() {
        assert_eq!(
            format!(
                "{}",
                Value::Fun {
                    params: vec!["a".to_string(), "b".to_string()],
                    body: Box::new(Expr::Literal {
                        value: Literal::None
                    }),
                    environment: Environment::new_top_level(),
                }
            ),
            "Value(Fun(params: [\"a\", \"b\"], body: Expr(None), environment: RefCell { value: Environment { enclosing: None, values: {} } }))".to_string()
        );
    }

    #[test]
    fn test_display_none() {
        assert_eq!(format!("{}", Value::None), "Value(none)");
    }
}
