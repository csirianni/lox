use crate::token::Token;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: BTreeMap<String, Value>,
}

impl Environment {
    pub fn new_top_level() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            enclosing: None,
            values: BTreeMap::new(),
        }))
    }

    pub fn new_block(environment: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(environment),
            values: BTreeMap::new(),
        }))
    }

    // TODO: Can we consume the name here?
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    /// Assigns a new value to `name`, if it exists, and returns the old value. Otherwise, does
    /// nothing and returns `None`.
    ///
    /// The key difference between assignment and definition is that assignment is not allowed to
    /// create a new variable.
    pub fn assign(&mut self, name: &Token, value: Value) -> Option<Value> {
        if let Some(entry) = self.values.get_mut(&name.lexeme) {
            let result = entry.clone();
            *entry = value;
            Some(result.clone())
        } else {
            match &mut self.enclosing {
                Some(enclosing) => enclosing.borrow_mut().assign(name, value),
                None => None,
            }
        }
    }

    pub fn get(&self, name: &Token) -> Option<Value> {
        if let Some(value) = self.values.get(&name.lexeme) {
            Some(value.clone())
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get(name),
                None => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Literal;
    use crate::token_type::TokenType;

    #[test]
    fn test_define() {
        let environment = Environment::new_top_level();
        environment.borrow_mut().define("foo", Value::Number(5_f64));
        let key = Token {
            token_type: TokenType::Identifier,
            lexeme: "foo".to_string(),
            literal: Literal::None,
            line: 0,
        };
        assert_eq!(environment.borrow().get(&key), Some(Value::Number(5_f64)));

        // Re-defining variables is allowed.
        environment.borrow_mut().define("foo", Value::Number(6_f64));
        assert_eq!(environment.borrow().get(&key), Some(Value::Number(6_f64)));
    }

    #[test]
    fn test_assign() {
        let environment = Environment::new_top_level();
        environment.borrow_mut().define("foo", Value::Number(5_f64));
        let foo = Token {
            token_type: TokenType::Identifier,
            lexeme: "foo".to_string(),
            literal: Literal::None,
            line: 0,
        };
        assert_eq!(environment.borrow().get(&foo), Some(Value::Number(5_f64)));

        assert_eq!(
            environment.borrow_mut().assign(&foo, Value::Number(6_f64)),
            Some(Value::Number(5_f64))
        );
        assert_eq!(environment.borrow().get(&foo), Some(Value::Number(6_f64)));

        // Undefined variable.
        let bar = Token {
            token_type: TokenType::Identifier,
            lexeme: "bar".to_string(),
            literal: Literal::None,
            line: 0,
        };
        assert_eq!(
            environment.borrow_mut().assign(&bar, Value::Number(1_f64)),
            None
        );
    }

    #[test]
    fn test_shadowing() {
        let tle = Environment::new_top_level();
        tle.borrow_mut().define("foo", Value::Number(5_f64));

        let block = Environment::new_block(tle);
        block.borrow_mut().define("foo", Value::Number(3_f64));
        let key = Token {
            token_type: TokenType::Identifier,
            lexeme: "foo".to_string(),
            literal: Literal::None,
            line: 0,
        };
        assert_eq!(block.borrow().get(&key), Some(Value::Number(3_f64)));
    }

    #[test]
    fn test_parent_lookup() {
        let tle = Environment::new_top_level();
        tle.borrow_mut().define("foo", Value::Number(5_f64));

        let block = Environment::new_block(tle);
        let key = Token {
            token_type: TokenType::Identifier,
            lexeme: "foo".to_string(),
            literal: Literal::None,
            line: 0,
        };
        assert_eq!(block.borrow().get(&key), Some(Value::Number(5_f64)));
    }

    #[test]
    fn test_parent_assign() {
        let tle = Environment::new_top_level();
        tle.borrow_mut().define("foo", Value::Number(5_f64));

        let block = Environment::new_block(tle);
        let key = Token {
            token_type: TokenType::Identifier,
            lexeme: "foo".to_string(),
            literal: Literal::None,
            line: 0,
        };
        block.borrow_mut().assign(&key, Value::Number(3_f64));
        assert_eq!(block.borrow().get(&key), Some(Value::Number(3_f64)));
    }
}
