mod rlox;
mod tokentype;
mod token;
mod scanner;

fn main() {
    let args = std::env::args().collect();
    rlox::main(args);
}