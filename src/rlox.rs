use std::io::Write;
use std::io::stdout;
use std::process::exit;
use crate::scanner::Scanner;

static mut HAD_ERROR: bool = false;

pub fn main(args: Vec<String>) {
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        exit(64);
    } else if args.len() == 2 {
        run_file(&args[1])
    } else {
        run_prompt()
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
                    unsafe {
                        if HAD_ERROR {
                            exit(65);
                        }
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
                unsafe {HAD_ERROR = false;}
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
    for token in tokens {
        println!("{:?}", token);
    }
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, location: &str, message: &str) {
    println!("[line {}] Error {}: {}", line, location, message);
    unsafe {HAD_ERROR = false;}
}
