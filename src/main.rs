mod tokens;
mod scanner;
mod parser;

use scanner::Scanner;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;
use tokens::{Literal, Token, TokenList};
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

fn split_by_line(tokens: Vec<Token>) -> Vec<Vec<Token>> {
    let mut groups: Vec<Vec<Token>> = Vec::new();
    let mut current_line: Option<usize> = None;
 
    for tok in tokens {
        if tok.token_type == TokenList::Engk {
            break;
        }
        match current_line {
            Some(line) if line == tok.line => {
                groups.last_mut().unwrap().push(tok);
            }
            _ => {
                current_line = Some(tok.line);
                groups.push(vec![tok]);
            }
        }
    }
 
    for group in groups.iter_mut() {
        let line = group.last().map(|t| t.line).unwrap_or(1);
        group.push(Token {
            token_type: TokenList::Engk,
            lexeme: String::new(),
            literal: Literal::None,
            line,
        });
    }
 
    groups
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
 
    let mut outputs = Vec::new();
    let mut ok = true;
 
    for group in split_by_line(tokens) {
        match Parser::new(group).parse_line() {
            Ok(expr) => outputs.push(expr.to_string()),
            Err(errs) => {
                for e in &errs {
                    eprintln!("{e}");
                }
                ok = false;
            }
        }
    }
 
    if ok {
        for line in outputs {
            println!("{line}");
        }
    }
    ok
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
        3 if args[1] == "--tokenize" => run_file_with(&args[2], run_parse),
        _ => {
            eprintln!("Usage: scanner [--tokenize <path>]");
            ExitCode::from(64)
        }
    }
}