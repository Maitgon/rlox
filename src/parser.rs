use crate::token::*;
use crate::expressions::*;
use crate::tokentype::*;
use crate::rlox::report;
use crate::statements::*;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    // Grammar for Lox
    // program -> declaration* EOF;
    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(message) => {
                    self.synchronize();
                    return Err(message);
                }
            }
        }

        Ok(statements)
    }

    // declaration -> varDecl | statement ;
    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(vec![TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    // varDecl -> "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = match self.peek().token_type {
            TokenType::Identifier(_) => {
                self.advance();
                self.previous()
            }
            _ => {
                return Err(String::from("Expect variable name."));
            }
        };
        let initializer = if self.match_token(vec![TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal(Token::new(TokenType::Nil, String::from("nil"), 0))
        };

        self.consume(TokenType::Semicolon, String::from("Expect ';' after variable declaration."))?;
        Ok(Stmt::Var(name, initializer))
    }

    // statement -> exprStmt | printStmt | block ;
    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(vec![TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(vec![TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    // block -> "{" declaration* "}" ;
    fn block(&mut self) -> Result<Stmt, String> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(message) => {
                    self.synchronize();
                    return Err(message);
                }
            }
        }

        self.consume(TokenType::RightBrace, String::from("Expect '}' after block."))?;
        Ok(Stmt::Block(statements))
    }

    // printStmt -> "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, String::from("Expect ';' after expression."))?;
        Ok(Stmt::Print(value))
    }

    // exprStmt -> expression ";" ;
    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, String::from("Expect ';' after expression."))?;
        Ok(Stmt::Expression(expr))
    }

    // Expressions grammar
    // expresion -> comma ;
    pub fn expression(&mut self) -> Result<Expr, String> {
        self.comma()
    }

    // comma -> assignment ( "," assignment )* ;
    fn comma(&mut self) -> Result<Expr, String> {
        let mut expr = self.assignment()?;

        while self.match_token(vec![TokenType::Comma]) {
            let operator = self.previous();
            let right = self.assignment()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    // assignment -> IDENTIFIER "=" assignment | ternary ;
    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.ternary()?;

        if self.match_token(vec![TokenType::Equal]) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name) => Ok(Expr::Assign(name, Box::new(value))),
                _ => Err(String::from("Invalid assignment target.")),
            }
        } else {
            Ok(expr)
        }
    }

    // ternary -> equality ( "?" equality ":" equality )? ;
    fn ternary(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        if self.match_token(vec![TokenType::QuestionMark]) {
            let operator1 = self.previous();
            let middle = self.equality()?;
            let operator2 = self.consume(TokenType::Colon, String::from("Expect ':' after expression."));
            match operator2 {
                Ok(_) => (),
                Err(message) => return Err(message),
            }
            let right = self.equality()?;
            expr = Expr::Ternary(Box::new(expr), operator1, Box::new(middle), operator2?, Box::new(right));
        }

        Ok(expr)
    }

    // equality -> comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    // comparison -> addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.addition()?;

        while self.match_token(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.addition()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    // addition -> multiplication ( ( "-" | "+" ) multiplication )* ;
    fn addition(&mut self) -> Result<Expr, String> {
        let mut expr = self.multiplication()?;

        while self.match_token(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.multiplication()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    // multiplication -> unary ( ( "/" | "*" ) unary )* ;
    fn multiplication(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_token(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    // unary -> ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.primary()
    }

    // primary -> NUMBER | STRING | "false" | "true" | "nil" | "(" expression ")" | IDENTIFIER;
    fn primary(&mut self) -> Result<Expr, String> {
        match self.peek().token_type {
            TokenType::False | TokenType::True | TokenType::Nil | TokenType::Number(_) | TokenType::String(_) => {
                self.advance();
                Ok(Expr::Literal(self.previous()))
            }
            TokenType::Identifier(_) => {
                self.advance();
                Ok(Expr::Variable(self.previous()))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                match self.consume(TokenType::RightParen, String::from("Expect ')' after expression.")) {
                    Ok(_) => Ok(Expr::Grouping(Box::new(expr))),
                    Err(message) => Err(message),
                }

            }
            _ => Err(String::from("Expect expression.")),
        }
    }

    // Error handling
    pub fn error(&mut self, token: Token, message: &str) {
        if token.token_type == crate::tokentype::TokenType::Eof {
            report(token.line, " at end", message);
        } else {
            report(token.line, format!(" at '{}'", token.lexeme).as_str(), message);
        }
    }

    pub fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }

    // Auxiliary functions for the parser
    fn match_token(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            self.error(self.peek(), message.as_str());
            Err(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::*;
    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(1.0), String::from("1"), 1),
            Token::new(TokenType::Plus, String::from("+"), 1),
            Token::new(TokenType::Number(2.0), String::from("2"), 1),
            Token::new(TokenType::Star, String::from("*"), 1),
            Token::new(TokenType::Number(3.0), String::from("3"), 1),
            Token::new(TokenType::Eof, String::from(""), 1),
        ];

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();
        let expr2 = expr.clone();

        if expr2.is_err() {
            println!("{}", expr2.err().unwrap());
        }

        assert_eq!(expr, Ok(Expr::Binary(
            Box::new(Expr::Literal(Token::new(TokenType::Number(1.0), String::from("1"), 1))),
            Token::new(TokenType::Plus, String::from("+"), 1),
            Box::new(Expr::Binary(
                Box::new(Expr::Literal(Token::new(TokenType::Number(2.0), String::from("2"), 1))),
                Token::new(TokenType::Star, String::from("*"), 1),
                Box::new(Expr::Literal(Token::new(TokenType::Number(3.0), String::from("3"), 1))
            )),
            )))
        );
    }

    #[test]
    fn test_parse_error() {
        let tokens = vec![
            Token::new(TokenType::Number(1.0), String::from("1"), 1),
            Token::new(TokenType::Plus, String::from("+"), 1),
            Token::new(TokenType::Number(2.0), String::from("2"), 1),
            Token::new(TokenType::Star, String::from("*"), 1),
            Token::new(TokenType::Eof, String::from(""), 1),
        ];

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        assert_eq!(expr, Err(String::from("Expect expression.")));
    }

    #[test]
    fn test_parse_and_scanned() {
        let source = "1 + 2 == 5 / 2";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        assert_eq!(expr, Ok(Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Literal(Token::new(TokenType::Number(1.0), String::from("1"), 1))),
                Token::new(TokenType::Plus, String::from("+"), 1),
                Box::new(Expr::Literal(Token::new(TokenType::Number(2.0), String::from("2"), 1)))
            )),
            Token::new(TokenType::EqualEqual, String::from("=="), 1),
            Box::new(Expr::Binary(
                Box::new(Expr::Literal(Token::new(TokenType::Number(5.0), String::from("5"), 1))),
                Token::new(TokenType::Slash, String::from("/"), 1),
                Box::new(Expr::Literal(Token::new(TokenType::Number(2.0), String::from("2"), 1)))
            ))
        )));
    }

    #[test]
    fn test_associative() {
        let source = "1 + 2 + 3";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        assert_eq!(expr, Ok(Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Literal(Token::new(TokenType::Number(1.0), String::from("1"), 1))),
                Token::new(TokenType::Plus, String::from("+"), 1),
                Box::new(Expr::Literal(Token::new(TokenType::Number(2.0), String::from("2"), 1)))
            )),
            Token::new(TokenType::Plus, String::from("+"), 1),
            Box::new(Expr::Literal(Token::new(TokenType::Number(3.0), String::from("3"), 1)))
        )));
    }

    #[test]
    fn test_parse_and_scanned_error() {
        let source = "1 + 2 == 5 /";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        assert_eq!(expr, Err(String::from("Expect expression.")));
    }

    #[test]
    fn test_parse_and_scanned_error_parenthesis() {
        let source = "1 + 2 == 5 / (2";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        assert_eq!(expr, Err(String::from("Expect ')' after expression.")));
    }

    #[test]
    fn test_parse_and_scanned_unary_and_primary() {
        let source = "-1 + aux == 5";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        assert_eq!(expr, Ok(Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Unary(
                    Token::new(TokenType::Minus, String::from("-"), 1),
                    Box::new(Expr::Literal(Token::new(TokenType::Number(1.0), String::from("1"), 1)))
                )),
                Token::new(TokenType::Plus, String::from("+"), 1),
                Box::new(Expr::Literal(Token::new(TokenType::Identifier(String::from("aux")), String::from("aux"), 1)))
                //Box::new(Expr::Literal(Token::new(TokenType::Number(2.0), String::from("2"), 1)))
            )),
            Token::new(TokenType::EqualEqual, String::from("=="), 1),
            Box::new(Expr::Literal(Token::new(TokenType::Number(5.0), String::from("5"), 1)))
        )));
    }

    #[test]
    fn test_parse_identifier() {
        let source = "aux";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        assert_eq!(expr, Ok(Expr::Literal(Token::new(TokenType::Identifier(String::from("aux")), String::from("aux"), 1))));
    }

    #[test]
    fn test_parse_comman() {
        let source = "1, 2, 3";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        assert_eq!(expr, Ok(Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Literal(Token::new(TokenType::Number(1.0), String::from("1"), 1))),
                Token::new(TokenType::Comma, String::from(","), 1),
                Box::new(Expr::Literal(Token::new(TokenType::Number(2.0), String::from("2"), 1)))
            )),
            Token::new(TokenType::Comma, String::from(","), 1),
            Box::new(Expr::Literal(Token::new(TokenType::Number(3.0), String::from("3"), 1)))
        )));
    }

    #[test]
    fn test_parse_comman_error() {
        let source = "1, 2, 3 +";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();
        assert_eq!(expr, Err(String::from("Expect expression.")));
    }

    #[test]
    fn test_parse_ternary() {
        let source = "5 ? 1 : 2";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();
        assert_eq!(expr, Ok(Expr::Ternary(
            Box::new(Expr::Literal(Token::new(TokenType::Number(5.0), String::from("5"), 1))),
            Token::new(TokenType::QuestionMark, String::from("?"), 1),
            Box::new(Expr::Literal(Token::new(TokenType::Number(1.0), String::from("1"), 1))),
            Token::new(TokenType::Colon, String::from(":"), 1),
            Box::new(Expr::Literal(Token::new(TokenType::Number(2.0), String::from("2"), 1)))
        )));
    }

    #[test]
    fn test_parse_ternary_error() {
        let source = "5 ? 1 + 2";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();
        assert_eq!(expr, Err(String::from("Expect ':' after expression.")));
    }

    #[test]
    fn test_program() {
        let source = "var a = 1; var b = 2; print a + b;";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();
        assert_eq!(statements, Ok(vec![
            Stmt::Var(Token::new(TokenType::Identifier(String::from("a")), String::from("a"), 1), Expr::Literal(Token::new(TokenType::Number(1.0), String::from("1"), 1))),
            Stmt::Var(Token::new(TokenType::Identifier(String::from("b")), String::from("b"), 1), Expr::Literal(Token::new(TokenType::Number(2.0), String::from("2"), 1))),
            Stmt::Print(Expr::Binary(
                Box::new(Expr::Variable(Token::new(TokenType::Identifier(String::from("a")), String::from("a"), 1))),
                Token::new(TokenType::Plus, String::from("+"), 1),
                Box::new(Expr::Variable(Token::new(TokenType::Identifier(String::from("b")), String::from("b"), 1)))
            ))
        ]));
    }

    #[test]
    fn test_program_error() {
        let source = "var a = 1; var b = 2; print a + b";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse(), Err(String::from("Expect ';' after expression.")));
    }

    #[test]
    fn test_program_error2() {
        let source = "var a = ;";

        let mut scanner = Scanner::new(String::from(source));
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse(), Err(String::from("Expect expression.")));
    }
}