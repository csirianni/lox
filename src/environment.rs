use crate::token::Token;
use crate::value::Value;
use std::collections::BTreeMap;

pub struct Environment {
    values: BTreeMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: BTreeMap::new(),
        }
    }

    // TODO: Can we consume the name here?
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Option<&Value> {
        self.values.get(&name.lexeme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Literal;
    use crate::token_type::TokenType;

    #[test]
    fn test_define() {
        let mut environment = Environment::new();
        environment.define("foo", Value::Number(5_f64));
        let key = Token {
            token_type: TokenType::Identifier,
            lexeme: "foo".to_string(),
            literal: Literal::None,
            line: 0,
        };
        assert_eq!(environment.get(&key), Some(&Value::Number(5_f64)));

        // Re-defining variables in allowed.
        environment.define("foo", Value::Number(6_f64));
        assert_eq!(environment.get(&key), Some(&Value::Number(6_f64)));
    }
}
