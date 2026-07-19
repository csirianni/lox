use std::env;
use std::io;

use crate::lox::Lox;

mod expr;
mod interpreter;
mod lox;
mod parser;
mod scanner;
mod token;
mod token_type;
mod value;

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
