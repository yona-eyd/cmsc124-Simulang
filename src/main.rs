mod tokens;
mod scanner;
mod parser;

use scanner::Scanner;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;
use parser::Parser;

fn run_tokenizer(source: &str) -> bool {
    let scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(tokens) => {
            for token in &tokens {
                println!("{token}");
            }
            true
        }
        Err(errors) => {
            for err in &errors {
                eprintln!("[line {}] Error: {}", err.line, err.message);
            }
            false
        }
    }
}
 
fn run_parse(source: &str) -> bool {
    let scanner = Scanner::new(source);
    let tokens = match scanner.scan_tokens() {
        Ok(t) => t,
        Err(errors) => {
            for err in &errors {
                eprintln!("[line {}] Error: {}", err.line, err.message);
            }
            return false;
        }
    };

    match Parser::new(tokens).parse(){
        Ok(exprs) => {
            for expr in &exprs {
                println!("{expr}");
            }
            true
        }
        Err(errs) => {
            for e in &errs {
                eprintln!("{e}");
            }
            false
        }
    }
}

fn run_file_with(path: &str, run: fn(&str)-> bool) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not read file '{}': {}", path, e);
            return ExitCode::from(70);
        }
    };
    if run(&source) {
        ExitCode::from(0)
    } else {
        ExitCode::from(65)  // sysexits, data format error     
    }
}

fn run_prompt() -> ExitCode {
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if stdin.read_line(&mut line).unwrap_or(0) == 0 {
            break; 
        }
        if line.trim().is_empty (){
            continue;
        }
        match Scanner::new(&line).scan_tokens() {
            Ok(tokens) => match Parser::new(tokens).parse_line() {
                    Ok(expr) => println!("{expr}"),
                    Err(errs) => {
                        for e in &errs {
                            eprintln!("{e}");
                        }
                }
            },
            Err(errors) => {
                for err in &errors {
                    eprintln!("[line {}] Error: {}", err.line, err.message);
                }
            }
        }
    }
    ExitCode::from(0)

}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1].ends_with("hello.src") {
        println!("Hello, world!");
        return ExitCode::from(0);
    }
    match args.len() {
        1 => run_prompt(),
        3 if args[1] == "--tokenize" => run_file_with(&args[2], run_tokenizer),
        3 if args[1] == "--parse" => run_file_with(&args[2], run_parse),
        _ => {
            eprintln!("Usage: scanner [--tokenize | --parse <path>]");
            ExitCode::from(64)
        }
    }
}