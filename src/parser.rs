use crate::Lox;
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
pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    lox: &'a mut Lox,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Self {
            tokens,
            current: 0,
            lox,
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        self.expression().ok()
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_types(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.match_types(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_types(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_types(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        return Ok(expr);
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
            Err(self.error(self.peek(), message))
        }
    }

    fn error(&mut self, token: Token, message: String) -> ParserError {
        self.lox.parser_error(token.clone(), message.clone());
        ParserError {
            token: token,
            message: message,
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_type::TokenType;

    #[test]
    fn test_malformed_equality() {
        let tokens = vec![
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(5_f64),
                line: 0,
            },
            // TODO: Our grammer does not support this token type yet. When we do, this should be a
            // *syntax* error.
            Token {
                token_type: TokenType::Equal,
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
                token_type: TokenType::EOF,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut lox = Lox::new();
        let mut parser = Parser::new(tokens, &mut lox);
        assert_eq!(
            parser.factor(),
            Ok(Expr::Literal {
                value: Literal::Number(5_f64)
            }),
        );
    }

    #[test]
    fn test_factor_one_unary() {
        let tokens = vec![
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(5_f64),
                line: 0,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut lox = Lox::new();
        let mut parser = Parser::new(tokens, &mut lox);
        assert_eq!(
            parser.factor(),
            Ok(Expr::Literal {
                value: Literal::Number(5_f64)
            },)
        );
    }

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
            Token {
                token_type: TokenType::EOF,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut lox = Lox::new();
        let mut parser = Parser::new(tokens, &mut lox);
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
            Token {
                token_type: TokenType::EOF,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut lox = Lox::new();
        let mut parser = Parser::new(tokens, &mut lox);
        assert_eq!(
            parser.factor(),
            // Left-associative implies 5 / 12 * 32 = (5 / 12) * 32.
            Ok(Expr::Binary {
                left: Box::new(Expr::Binary {
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
                }),
                operator: Token {
                    token_type: TokenType::Star,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    value: Literal::Number(32_f64)
                }),
            })
        );
    }
}
