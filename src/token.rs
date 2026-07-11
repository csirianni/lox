use crate::token_type::TokenType;

#[derive(Clone, Debug)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    None,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}
