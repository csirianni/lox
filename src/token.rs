use crate::token_type::TokenType;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),

    #[default]
    None,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}
