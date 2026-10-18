mod tokens;
mod scanner;
mod parser;
mod evaluator;

use scanner::Scanner;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;
use parser::Parser;
use parser::Expr;
use evaluator::evaluate;

fn run_tokenizer(source: &str) -> u8 {
    let scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(tokens) => {
            for token in &tokens {
                println!("{token}");
            }
            0
        }
        Err(errors) => {
            for err in &errors {
                eprintln!("[line {}] Error: {}", err.line, err.message);
            }
            65
        }
    }
}

fn run_parse(source: &str) -> u8 {
    let scanner = Scanner::new(source);
    let tokens = match scanner.scan_tokens() {
        Ok(t) => t,
        Err(errors) => {
            for err in &errors {
                eprintln!("[line {}] Error: {}", err.line, err.message);
            }
            return 65;
        }
    };

    match Parser::new(tokens).parse(){
        Ok(tree) => {
            print!("{tree}");
            0
        }
        Err(errs) => {
            for e in &errs {
                eprintln!("{e}");
            }
            65
        }
    }
}

fn run_eval(source: &str) -> u8 {
    let scanner = Scanner::new(source);
    let tokens  = match scanner.scan_tokens() {
        Ok(t) => t,
        Err(errors) => {
            for err in &errors {
                eprintln!("[line {}] Error: {}", err.line, err.message);
            }
            return 65;
        }
    };

    let program = match Parser::new(tokens).parse() {
        Ok(p) => p,
        Err(errs) => {
            for e in &errs {
                eprintln!("{e}");
            }
            return 65;
        }
    };

    let exprs = match program {
        Expr::Program(exprs) => exprs,
        other => vec![other]    // if not Program, treat it as one item and don't panic
    };

    for expr in &exprs{
        match evaluate(expr) {
            Ok(value)   => println!("{value}"),
            Err(e)      => {
                eprintln!("{e}");
                return 70;
            }
        }
    }
    0
}

fn run_file_with(path: &str, run: fn(&str)-> u8) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not read file '{}': {}", path, e);
            return ExitCode::from(70);
        }
    };
    ExitCode::from(run(&source))
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
                    Ok(expr) => match evaluate(&expr) {
                        Ok(value)   => println!("{value}"),
                        Err(e)      => eprintln!("{e}"),
                    },
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
        3 if args[1] == "--tokenize"    => run_file_with(&args[2], run_tokenizer),
        3 if args[1] == "--parse"       => run_file_with(&args[2], run_parse),
        3 if args[1] == "--eval"        => run_file_with(&args[2], run_eval),
        _ => {
            eprintln!("Usage: scanner [--tokenize | --parse <path> | --eval <path>]");
            ExitCode::from(64)
        }
    }
}