use crate::expressions::Expr;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Expr),
    Block(Vec<Stmt>),
    //If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    //While(Expr, Box<Stmt>),
    //Function(Token, Vec<Token>, Vec<Stmt>),
    //Return(Token, Option<Expr>),
}