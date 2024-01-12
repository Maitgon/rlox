use std::fs;
use std::io::Write;

pub fn generate_ast(args: Vec<String>) {
    if args.len() != 2 {
        panic!("Usage: generate_ast <output directory>");
    }
    let output_dir = &args[1];
    define_ast(output_dir.to_string(), "Expr".to_string(), vec![
        "Binary   : Box<Expr> left, Token operator, Box<Expr> right".to_string(),
        "Grouping : Box<Expr> expression".to_string(),
        "Literal  : Object value".to_string(),
        "Unary    : Token operator, Box<Expr> right".to_string(),
    ]);
}

fn define_ast(output_dir: String, base_name: String, types: Vec<String>) {
    let path = format!("{}/{}.rs", output_dir, base_name);

    let mut file = fs::File::create(path).unwrap();

    file.write_all(b"use crate::token::Token;\n").unwrap();
}