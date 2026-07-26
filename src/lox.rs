use std::fs;
use std::io::{self, Write};

use crate::interpreter::{RuntimeError, interpret};
use crate::parser::{Parser, ParserError};
use crate::scanner::Scanner;
use crate::token::Token;
use crate::token_type::TokenType;

pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn run_file(&mut self, path: &str) -> io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.run(content);

        if self.had_error {
            std::process::exit(65);
        }

        if self.had_runtime_error {
            std::process::exit(70);
        }

        Ok(())
    }

    pub fn run_prompt(&mut self) -> io::Result<()> {
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            if line.trim_end().is_empty() {
                return Ok(());
            }
            self.run(line);
            // If the user makes a mistake, it shouldn’t kill their entire session.
            self.had_error = false;
        }
    }

    fn run(&mut self, line: String) {
        let mut scanner = Scanner::new(line.to_owned());
        let tokens = scanner.scan_tokens(self);

        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(statements) => {
                // FIX: The environment is per-line right now. We need the environment to outlive
                // this function.
                if let Err(error) = interpret(statements) {
                    self.interpreter_error(error)
                }
            }
            Err(ParserError { token, message }) => {
                self.parser_error(token, message);
            }
        }
    }

    pub fn scanner_error(&mut self, line: usize, message: String) {
        self.report(line, "".to_string(), message);
    }

    fn parser_error(&mut self, token: Token, message: String) {
        if token.token_type == TokenType::Eof {
            self.report(token.line, " at end".to_string(), message);
        } else {
            self.report(token.line, format!(" at '{}'", token.lexeme), message);
        }
    }

    fn report(&mut self, line: usize, location: String, message: String) {
        println!("[line {}] Error{}: {}", line, location, message);
        self.had_error = true;
    }

    fn interpreter_error(&mut self, error: RuntimeError) {
        eprintln!("{} \n[line {}]", error.message, error.token.line);
        self.had_runtime_error = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_error() {
        let mut lox = Lox::new();
        // No semicolon.
        lox.run("print 1".to_string());
        assert!(lox.had_error);
        assert!(!lox.had_runtime_error);
    }

    #[test]
    fn test_runtime_error() {
        let mut lox = Lox::new();
        // Runtime type error.
        lox.run("1 + false;".to_string());
        assert!(!lox.had_error);
        assert!(lox.had_runtime_error);
    }

    #[test]
    fn test_statement() {
        let mut lox = Lox::new();
        lox.run("print 1 + 2;".to_string());
        assert!(!lox.had_error);
        assert!(!lox.had_runtime_error);
    }
}
