use crate::expr::Expr;
use crate::token::{Literal, Token};
use crate::token_type::TokenType;

type Result<T> = std::result::Result<T, ParserError>;

#[derive(Debug, Clone, PartialEq)]
struct ParserError {
    token: Token,
    message: String,
}

/// https://craftinginterpreters.com/parsing-expressions.html
/// expression     → equality ;
/// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
/// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term           → factor ( ( "-" | "+" ) factor )* ;
/// factor         → unary ( ( "/" | "*" ) unary )* ;
/// unary          → ( "!" | "-" ) unary
///                | primary ;
/// primary        → NUMBER | STRING | "true" | "false" | "nil"
///                | "(" expression ")" ;
struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        self.expression().ok()
    }

    fn expression(&mut self) -> Result<Expr> {
        unimplemented!();
    }

    fn term(&mut self) -> Result<Expr> {
        unimplemented!();
    }

    fn comparison(&mut self) -> Result<Expr> {
        unimplemented!();
    }

    fn factor(&mut self) -> Result<Expr> {
        let left = self.unary()?;

        if self.match_types(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            })
        } else {
            self.unary()
        }
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_types(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_types(&[TokenType::False]) {
            Ok(Expr::Literal {
                value: Literal::Boolean(false),
            })
        } else if self.match_types(&[TokenType::True]) {
            Ok(Expr::Literal {
                value: Literal::Boolean(true),
            })
        } else if self.match_types(&[TokenType::Nil]) {
            Ok(Expr::Literal {
                value: Literal::None,
            })
        } else if self.match_types(&[TokenType::Number, TokenType::Str]) {
            Ok(Expr::Literal {
                value: self.previous().literal,
            })
        } else if self.match_types(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                "Expected ')' after expression".to_string(),
            )?;
            Ok(Expr::Grouping {
                expression: Box::new(expr),
            })
        } else {
            unreachable!();
        }
    }

    fn match_types(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            // TODO: Update Lox error.
            Err(ParserError {
                token: self.peek(),
                message: message,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_type::TokenType;

    #[test]
    fn test_factor_two_unary() {
        let tokens = vec![
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(5_f64),
                line: 0,
            },
            Token {
                token_type: TokenType::Slash,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(12_f64),
                line: 0,
            },
        ];
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.factor(),
            Ok(Expr::Binary {
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
                    value: Literal::Number(12_f64)
                })
            })
        );
    }

    #[test]
    fn test_factor_three_unary() {
        let tokens = vec![
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(5_f64),
                line: 0,
            },
            Token {
                token_type: TokenType::Slash,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(12_f64),
                line: 0,
            },
            Token {
                token_type: TokenType::Star,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(32_f64),
                line: 0,
            },
        ];
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.factor(),
            Ok(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(5_f64)
                }),
                operator: Token {
                    token_type: TokenType::Slash,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(12_f64)
                    }),
                    operator: Token {
                        token_type: TokenType::Star,
                        lexeme: "".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(32_f64)
                    })
                })
            })
        );
    }
}
