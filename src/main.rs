mod rlox;
mod tokentype;
mod token;
mod scanner;
mod tools;
mod expressions;
mod parser;
mod interpreter;

fn main() {
    let args = std::env::args().collect();
    rlox::main(args);
}