use std::env;
use std::fs;
use std::io::{self, Write};

use crate::scanner::Scanner;

mod expr;
mod parser;
mod scanner;
mod token;
mod token_type;

fn main() -> io::Result<()> {
    let mut lox = Lox::new();
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: jlox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        lox.run_file(&args[0])?;
    } else {
        lox.run_prompt()?;
    }
    Ok(())
}

struct Lox {
    had_error: bool,
}

impl Lox {
    fn new() -> Self {
        Lox { had_error: false }
    }

    fn run_file(&mut self, path: &str) -> io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.run(content);

        if self.had_error {
            std::process::exit(65);
        }

        Ok(())
    }

    fn run_prompt(&mut self) -> io::Result<()> {
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            if line == "" {
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

        for token in tokens {
            println!("{:?}", token);
        }
    }

    fn error(&mut self, line: usize, message: String) {
        self.report(line, "".to_string(), message);
    }

    fn report(&mut self, line: usize, location: String, message: String) {
        println!("[line {}] Error{}: {}", line, location, message);
        self.had_error = true;
    }
}
