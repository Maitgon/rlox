use std::fmt;
use crate::tokentype::*;
use crate::expressions::*;
use crate::statements::*;
use crate::environment::*;

pub struct Interpreter {
    pub had_error: bool,
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            had_error: false,
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), String> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: Stmt) -> Result<(), String> {
        match statement {
            Stmt::Expression(expression) => {
                self.evaluate_expression(expression)?;
            }
            Stmt::Print(expression) => {
                let value = self.evaluate_expression(expression)?;
                println!("{}", value);
            }
            Stmt::Var(name, expression) => {
                let value = self.evaluate_expression(expression)?;
                self.environment.define(name.lexeme, value);
            }
            Stmt::Block(statements) => {
                let previous = self.environment.clone();
                self.environment.enclosing = Some(Box::new(previous.clone()));
                for statement in statements {
                    self.execute_statement(statement)?;
                }
                self.environment = previous;
            }
        }
        Ok(())
    }

    fn evaluate_expression(&mut self, expression: Expr) -> Result<Value, String> {
        match expression {

            // Literal evaluation
            Expr::Literal(token) => {
                match token.token_type {
                    TokenType::Number(number) => Ok(Value::Number(number)),
                    TokenType::String(string) => Ok(Value::String(string)),
                    TokenType::True => Ok(Value::Boolean(true)),
                    TokenType::False => Ok(Value::Boolean(false)),
                    TokenType::Nil => Ok(Value::Nil),
                    TokenType::Identifier(name) => self.environment.get(&name),
                    _ => Err(format!("Unexpected token type: '{}' for Literal Expresion", token.token_type)),
                }
            }

            Expr::Variable(name) => {
                self.environment.get(&name.lexeme)
            }

            // Grouping / Parenthesis evaluation
            Expr::Grouping(expression) => self.evaluate_expression(*expression),

            // Unary evaluation
            Expr::Unary(operator, right) => {
                let right = self.evaluate_expression(*right)?;
                match operator.token_type {
                    TokenType::Minus => {
                        match right {
                            Value::Number(number) => Ok(Value::Number(-number)),
                            _ => Err(format!("Unexpected value: '{}' for Unary Expression: -{}", right, right)),
                        }
                    }
                    TokenType::Bang => {
                        Ok(Value::Boolean(!self.is_truthy(right)))
                    }
                    _ => Err(format!("Unexpected token type: '{}' for Unary Expression", operator.token_type)),
                }
            }

            // Binary evaluation
            Expr::Binary(left, operator, right) => {
                let left = self.evaluate_expression(*left)?;
                let right = self.evaluate_expression(*right)?;
                match operator.token_type {

                    // Comma expressions
                    TokenType::Comma => {
                        Ok(right)
                    }

                    // Equality expressions
                    TokenType::EqualEqual => {
                        Ok(Value::Boolean(left == right))
                    }
                    TokenType::BangEqual => {
                        Ok(Value::Boolean(left != right))
                    }

                    // Comparison expressions
                    TokenType::Greater | TokenType::Less | TokenType::GreaterEqual | TokenType::LessEqual => {
                        match (&left, &right) {
                            (Value::Number(left), Value::Number(right)) => {
                                match operator.token_type {
                                    TokenType::Greater => Ok(Value::Boolean(left > right)),
                                    TokenType::Less => Ok(Value::Boolean(left < right)),
                                    TokenType::GreaterEqual => Ok(Value::Boolean(left >= right)),
                                    TokenType::LessEqual => Ok(Value::Boolean(left <= right)),
                                    _ => Err(format!("Unexpected token type: '{}' for Binary Expression", operator.token_type)),
                                }
                            }
                            _ => Err(format!("Unexpected values: '{}' and '{}' for Binary Expression: {} {} {}", left, right, left, operator.token_type, right)),
                        }
                    }

                    // Arithmetic expressions
                    TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash => {
                        match (&left, &right) {
                            (Value::Number(left), Value::Number(right)) => {
                                match operator.token_type {
                                    TokenType::Plus => Ok(Value::Number(left + right)),
                                    TokenType::Minus => Ok(Value::Number(left - right)),
                                    TokenType::Star => Ok(Value::Number(left * right)),
                                    TokenType::Slash => {
                                        if right == &0.0 {
                                            Err(format!("Division by zero: {} {} {}", left, operator.token_type, right))
                                        } else {
                                            Ok(Value::Number(left / right))
                                        }
                                    }
                                    _ => Err(format!("Unexpected token type: '{}' for Binary Expression", operator.token_type)),
                                }
                            }
                            (Value::String(left), Value::String(right)) => {
                                match operator.token_type {
                                    TokenType::Plus => Ok(Value::String(format!("{}{}", left, right))),
                                    _ => Err(format!("Unexpected token type: '{}' for Binary Expression", operator.token_type)),
                                }
                            }
                            (left, Value::String(right)) => {
                                match operator.token_type {
                                    TokenType::Plus => Ok(Value::String(format!("{}{}", left, right))),
                                    _ => Err(format!("Unexpected token type: '{}' for Binary Expression", operator.token_type)),
                                }
                            }
                            (Value::String(left), right) => {
                                match operator.token_type {
                                    TokenType::Plus => Ok(Value::String(format!("{}{}", left, right))),
                                    _ => Err(format!("Unexpected token type: '{}' for Binary Expression", operator.token_type)),
                                }
                            }
                            _ => Err(format!("Unexpected values: '{}' and '{}' for Binary Expression: {} {} {}", left, right, left, operator.token_type, right)),
                        }
                    }

                    _ => Err(format!("Unexpected token type: '{}' for Binary Expression", operator.token_type)),
                }
            }

            // Ternary evaluation
            Expr::Ternary(left, operator1, middle, operator2, right) => {
                let left = self.evaluate_expression(*left)?;
                let middle = self.evaluate_expression(*middle)?;
                let right = self.evaluate_expression(*right)?;
                match operator1.token_type {
                    TokenType::QuestionMark => {
                        match operator2.token_type {
                            TokenType::Colon => {
                                if self.is_truthy(left) {
                                    Ok(middle)
                                } else {
                                    Ok(right)
                                }
                            }
                            _ => Err(format!("Unexpected token type: '{}' for Ternary Expression: {} {} {} {} {}", operator2.token_type, left, operator1.token_type, middle, operator2.token_type, right)),
                        }
                    }
                    _ => Err(format!("Unexpected token type: '{}' for Ternary Expression: {} {} {} {} {}", operator1.token_type, left, operator1.token_type, middle, operator2.token_type, right)),
                }
            }

            // Assignment evaluation
            Expr::Assign(name, value) => {
                let new_val = self.evaluate_expression(*value)?;
                self.environment.assign(name.lexeme, new_val.clone())?;
                Ok(new_val)
            }
        }
    }

    fn is_truthy(&self, value: Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(boolean) => boolean,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(number) => write!(f, "{}", number),
            Value::String(string) => write!(f, "{}", string),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "nil"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::scanner::Scanner;

    fn get_result_from_expression(expression: &str) -> Result<Value, String> {
        let mut scanner = Scanner::new(String::from(expression));
        let mut parser = Parser::new(scanner.scan_tokens());
        let mut interpreter = Interpreter::new();

        let expression = parser.expression();
        match expression {
            Ok(expression) => interpreter.evaluate_expression(expression),
            Err(error) => Err(error),
        }
    }

    #[test]
    fn test_evaluate_literals_expression() {
        assert_eq!(get_result_from_expression("true"), Ok(Value::Boolean(true)));
        assert_eq!(get_result_from_expression("false"), Ok(Value::Boolean(false)));
        assert_eq!(get_result_from_expression("nil"), Ok(Value::Nil));
        assert_eq!(get_result_from_expression("1"), Ok(Value::Number(1.0)));
        assert_eq!(get_result_from_expression("1.5"), Ok(Value::Number(1.5)));
        assert_eq!(get_result_from_expression("\"Hello\""), Ok(Value::String(String::from("Hello"))));
    }

    #[test]
    fn test_evaluate_grouping_expression() {
        assert_eq!(get_result_from_expression("(1)"), Ok(Value::Number(1.0)));
        assert_eq!(get_result_from_expression("(1 + 2)"), Ok(Value::Number(3.0)));
        assert_eq!(get_result_from_expression("(1 + 2) * 3"), Ok(Value::Number(9.0)));
        assert_eq!(get_result_from_expression("1 + (2 * 3)"), Ok(Value::Number(7.0)));
    }

    #[test]
    fn test_evaluate_unary_expression() {
        assert_eq!(get_result_from_expression("-1"), Ok(Value::Number(-1.0)));
        assert_eq!(get_result_from_expression("!true"), Ok(Value::Boolean(false)));
        assert_eq!(get_result_from_expression("!false"), Ok(Value::Boolean(true)));
        assert_eq!(get_result_from_expression("!!true"), Ok(Value::Boolean(true)));
        assert_eq!(get_result_from_expression("!!false"), Ok(Value::Boolean(false)));
        assert_eq!(get_result_from_expression("!!1"), Ok(Value::Boolean(true)));
        assert_eq!(get_result_from_expression("!!nil"), Ok(Value::Boolean(false)));
    }

    #[test]
    fn test_evaluate_binary_numbers_expression() {
        assert_eq!(get_result_from_expression("1 + 2"), Ok(Value::Number(3.0)));
        assert_eq!(get_result_from_expression("1 - 2"), Ok(Value::Number(-1.0)));
        assert_eq!(get_result_from_expression("1 * 2"), Ok(Value::Number(2.0)));
        assert_eq!(get_result_from_expression("1 / 2"), Ok(Value::Number(0.5)));
        assert_eq!(get_result_from_expression("1 + 2 * 3"), Ok(Value::Number(7.0)));
        assert_eq!(get_result_from_expression("(1 + 2) * 3"), Ok(Value::Number(9.0)));
        assert_eq!(get_result_from_expression("1 + 2 * 3 + 4 / 2"), Ok(Value::Number(9.0)));
        assert_eq!(get_result_from_expression("1 + 1 + (2 + 3) + 5 + (8 + 13)"), Ok(Value::Number(33.0)));
    }

    #[test]
    fn test_division_by_zero_error() {
        assert_eq!(get_result_from_expression("1 / 0"), Err(String::from("Division by zero: 1 / 0")));
    }

    #[test]
    fn test_evaluate_binary_bool_expression() {
        assert_eq!(get_result_from_expression("true == false"), Ok(Value::Boolean(false)));
        assert_eq!(get_result_from_expression("true != false"), Ok(Value::Boolean(true)));
        assert_eq!(get_result_from_expression("true == true"), Ok(Value::Boolean(true)));
        assert_eq!(get_result_from_expression("true != true"), Ok(Value::Boolean(false)));
        assert_eq!(get_result_from_expression("1 + 2 == 6 / 2"), Ok(Value::Boolean(true)));
        assert_eq!(get_result_from_expression("!(2 * 3 != 2 + 2 + 2)"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn test_evaluate_binary_strings_expression() {
        assert_eq!(get_result_from_expression("\"Hello\" + \"World\""), Ok(Value::String(String::from("HelloWorld"))));
        assert_eq!(get_result_from_expression("\"Hello\" + \" \" + \"World\""), Ok(Value::String(String::from("Hello World"))));
    }

    #[test]
    fn test_evaluate_binary_comma_expression() {
        assert_eq!(get_result_from_expression("1, 2, 3"), Ok(Value::Number(3.0)));
        assert_eq!(get_result_from_expression("1, 2, 3, 4, 5"), Ok(Value::Number(5.0)));
        assert_eq!(get_result_from_expression("1 + 2, 3 / 5, 5 / 2"), Ok(Value::Number(2.5)));
    }

    #[test]
    fn test_comma_error_left() {
        assert_eq!(get_result_from_expression("3 / 0, 2 + 3"), Err(String::from("Division by zero: 3 / 0")));
    }

    #[test]
    fn test_comma_error_right() {
        assert_eq!(get_result_from_expression("2 + 3, 3 / 0"), Err(String::from("Division by zero: 3 / 0")));
    }

    #[test]
    fn test_ternary_expression() {
        assert_eq!(get_result_from_expression("true ? 1 : 2"), Ok(Value::Number(1.0)));
        assert_eq!(get_result_from_expression("false ? 1 : 2"), Ok(Value::Number(2.0)));
        assert_eq!(get_result_from_expression("1 != 2 ? 1+2 : 2-1"), Ok(Value::Number(3.0)));
        assert_eq!(get_result_from_expression("1 == 2 ? 1+2 : 2-1"), Ok(Value::Number(1.0)));
    }

    #[test]
    fn test_ternary_error() {
        assert_eq!(get_result_from_expression("1 == 2 ? 1/0 : 2+3"), Err(String::from("Division by zero: 1 / 0")));
    }

    #[test]
    fn test_error_initialized_variable() {
        assert_eq!(get_result_from_expression("a = 1"), Err(String::from("Undefined variable 'a'.")));
    }
}