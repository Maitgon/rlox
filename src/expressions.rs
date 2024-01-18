use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Ternary(Box<Expr>, Token, Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token),
    Unary(Token, Box<Expr>),
    Assign(Token, Box<Expr>),
    Variable(Token),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Binary(left, operator, right) => {
                write!(f, "({} {} {})", operator, **left, **right)
            }
            Expr::Ternary(left, operator1, middle, operator2, right) => {
                write!(f, "({} {} {} {} {})", operator1, **left, **middle, operator2, **right)
            }
            Expr::Grouping(expression) => write!(f, "(group {})", **expression),
            Expr::Literal(value) => write!(f, "{}", value),
            Expr::Unary(operator, right) => write!(f, "({} {})", operator, **right),
            Expr::Assign(name, value) => write!(f, "(assign {} {})", name, **value),
            Expr::Variable(name) => write!(f, "{}", name),
        }
    }
}