use std::io::Write;
use std::io::stdout;
use std::process::exit;
use crate::scanner::Scanner;
use crate::parser::Parser;
use std::sync::Mutex;

pub static HAD_ERROR: Mutex<bool> = Mutex::new(false);

pub fn main(args: Vec<String>) {
    match args.len().cmp(&2) { // Clippy wasn't happy with using if else :/
        std::cmp::Ordering::Greater => {
            println!("Usage: rlox [script]");
            exit(64);
        }
        std::cmp::Ordering::Equal => run_file(&args[1]),
        std::cmp::Ordering::Less => run_prompt(),
    }
}

fn run_file(path: &str) {
    let bytes = std::fs::read(path).ok();
    match bytes {
        Some(bytes) => {
            let source = String::from_utf8(bytes).ok();
            match source {
                Some(source) => {
                    run(source);
                    if *HAD_ERROR.lock().unwrap() {
                        exit(65);
                    }
                },
                None => {
                    println!("Error reading file: {}", path);
                    exit(66);
                }
            }
        },
        None => {
            println!("Error reading file: {}", path);
            exit(66);
        }
    }
}

fn run_prompt() {
    let reader = std::io::stdin();
    loop {
        print!("> ");
        stdout().flush().ok();
        let mut line = String::new();
        let res = reader.read_line(&mut line);
        match res {
            Ok(_) => {
                run(line);
                *HAD_ERROR.lock().unwrap() = false;
            },
            Err(_) => {
                println!("Error reading line");
                exit(66);
            }
        }
    }
}

fn run(source: String) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse();
    
    match statements {
        Ok(statements) => {
            println!("{}", statements);
        },
        Err(err) => {
            *HAD_ERROR.lock().unwrap() = true;
            println!("{}", err);
        }
    }
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn report(line: usize, location: &str, message: &str) {
    println!("[line {}] Error {}: {}", line, location, message);
    *HAD_ERROR.lock().unwrap() = true;
}
