use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Ternary(Box<Expr>, Token, Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token),
    Unary(Token, Box<Expr>),
}

impl Expr {
    pub fn new_binary(left: Expr, operator: Token, right: Expr) -> Expr {
        Expr::Binary(Box::new(left), operator, Box::new(right))
    }

    pub fn new_grouping(expression: Expr) -> Expr {
        Expr::Grouping(Box::new(expression))
    }

    pub fn new_literal(value: Token) -> Expr {
        Expr::Literal(value)
    }

    pub fn new_unary(operator: Token, right: Expr) -> Expr {
        Expr::Unary(operator, Box::new(right))
    }
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
        }
    }
}