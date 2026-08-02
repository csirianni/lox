use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token};
use crate::token_type::TokenType;

type Result<T> = std::result::Result<T, ParserError>;

#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    pub token: Token,
    pub message: String,
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
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::<Stmt>::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let statement = if self.match_types(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        if statement.is_err() {
            // TODO: Test this.
            self.synchronize();
        }
        statement
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name".to_string())?;
        let initializer = if self.match_types(&[TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal {
                value: Literal::None,
            }
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration".to_string(),
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_types(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_types(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block {
                statements: self.block()?,
            })
        } else if self.match_types(&[TokenType::If]) {
            self.if_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value".to_string())?;
        Ok(Stmt::Print { expression: value })
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block".to_string())?;
        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        let keyword = self.previous();
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'".to_string())?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            "Expect ')' after if condition".to_string(),
        )?;

        let consq = self.statement()?;
        if self.match_types(&[TokenType::Else]) {
            let altern = self.statement()?;
            Ok(Stmt::If {
                keyword,
                condition,
                consq: Box::new(consq),
                altern: Some(Box::new(altern)),
            })
        } else {
            Ok(Stmt::If {
                keyword,
                condition,
                consq: Box::new(consq),
                altern: None,
            })
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after expression".to_string(),
        )?;
        Ok(Stmt::Expression { expression: value })
    }

    /// expression → assignment ;
    /// assignment → IDENTIFIER "=" assignment
    ///            | equality ;
    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.equality()?;

        if self.match_types(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            return Err(self.error(equals, "Invalid assignment target".to_string()));
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
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

        Ok(expr)
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

        Ok(expr)
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

        Ok(expr)
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

        Ok(expr)
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
        } else if self.match_types(&[TokenType::Identifier]) {
            Ok(Expr::Variable {
                name: self.previous(),
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
        self.peek().token_type == TokenType::Eof
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
        ParserError { token, message }
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
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut parser = Parser::new(tokens);
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
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut parser = Parser::new(tokens);
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
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
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
            Token {
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut parser = Parser::new(tokens);
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

    #[test]
    fn test_assignment() {
        // a = 4;
        let tokens = vec![
            Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Equal,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(4_f64),
                line: 0,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.assignment(),
            Ok(Expr::Assign {
                name: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "a".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                value: Box::new(Expr::Literal {
                    value: Literal::Number(4_f64)
                }),
            })
        );
    }

    #[test]
    fn test_nested_assignment() {
        //  Nested assignment is not a syntax error. The value of an assignment expression is the
        //  rhs, so we allow the following program: a = b = 4;
        let tokens = vec![
            Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Equal,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: "b".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Equal,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: "".to_string(),
                literal: Literal::Number(4_f64),
                line: 0,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: 0,
            },
        ];
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.assignment(),
            Ok(Expr::Assign {
                name: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "a".to_string(),
                    literal: Literal::None,
                    line: 0,
                },
                value: Box::new(Expr::Assign {
                    name: Token {
                        token_type: TokenType::Identifier,
                        lexeme: "b".to_string(),
                        literal: Literal::None,
                        line: 0,
                    },
                    value: Box::new(Expr::Literal {
                        value: Literal::Number(4_f64)
                    }),
                })
            })
        );
    }
}
