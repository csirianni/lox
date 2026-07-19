use crate::expr::Expr;
use crate::token::Literal;
use crate::value::Value;

pub struct Interpreter {}

impl Interpreter {
    pub fn interpret(&self, expression: Expr) -> Value {
        match expression {
            Expr::Literal { value } => match value {
                Literal::String(str) => Value::String(str),
                Literal::Number(num) => Value::Number(num),
                Literal::Boolean(bool) => Value::Boolean(bool),
                Literal::None => Value::None,
            },
            _ => todo!(),
        }
    }
}
