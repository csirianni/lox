use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    None,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(str) => write!(f, "Value({:?})", str),
            Value::Number(num) => write!(f, "Value({:?})", num),
            Value::Boolean(bool) => write!(f, "Value({:?})", bool),
            Value::None => write!(f, "Value(none)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_display_none() {
        assert_eq!(format!("{}", Value::None), "Value(none)");
    }
}
