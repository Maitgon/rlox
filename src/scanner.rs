use crate::token::Token;
use crate::tokentype::TokenType;
use crate::rlox;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(TokenType::Eof, String::from(""), self.line));
        self.tokens.clone()
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            // Single character tokens
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            // One or two character tokens
            '!' => {
                let token_type = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type);
            },

            '=' => {
                let token_type = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type);
            },

            '>' => {
                let token_type = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type);
            }

            '<' => {
                let token_type = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token_type);
            }

            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    while self.peek() != '*' && self.peek_next() != '/' && !self.is_at_end() {
                        if self.peek() == '\n' {
                            self.line += 1;
                        }
                        self.advance();
                    }
                    if self.is_at_end() {
                        rlox::error(self.line, "Unterminated block comment");
                        return;
                    }
                    self.advance();
                    self.advance();
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            // Ignore whitespace
            ' ' | '\r' | '\t' => (),

            // Newline increases line number and is ignored
            '\n' => self.line += 1,

            // String literals
            '"' => self.string(),

            c => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_ascii_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    rlox::error(self.line, format!("Unexpected character: {}", c).as_str())
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, text, self.line));
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            rlox::error(self.line, "Unterminated string");
            return;
        }

        self.advance();
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::String(value));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current].parse::<f64>().unwrap();
        self.add_token(TokenType::Number(value));
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();
        // Here we match the identifier against the reserved words
        let token_type = match text.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier(text),
        };
        self.add_token(token_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_char_tokens() {
        let mut scanner = Scanner::new(String::from("(){},.-+;*/"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 12);
        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::RightParen);
        assert_eq!(tokens[2].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[3].token_type, TokenType::RightBrace);
        assert_eq!(tokens[4].token_type, TokenType::Comma);
        assert_eq!(tokens[5].token_type, TokenType::Dot);
        assert_eq!(tokens[6].token_type, TokenType::Minus);
        assert_eq!(tokens[7].token_type, TokenType::Plus);
        assert_eq!(tokens[8].token_type, TokenType::Semicolon);
        assert_eq!(tokens[9].token_type, TokenType::Star);
        assert_eq!(tokens[10].token_type, TokenType::Slash);
        assert_eq!(tokens[11].token_type, TokenType::Eof);
    }

    #[test]
    fn test_one_or_two_char_tokens() {
        let mut scanner = Scanner::new(String::from("!= == >= <= < >"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].token_type, TokenType::BangEqual);
        assert_eq!(tokens[1].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[2].token_type, TokenType::GreaterEqual);
        assert_eq!(tokens[3].token_type, TokenType::LessEqual);
        assert_eq!(tokens[4].token_type, TokenType::Less);
        assert_eq!(tokens[5].token_type, TokenType::Greater);
        assert_eq!(tokens[6].token_type, TokenType::Eof);
    }

    #[test]
    fn test_whitespace() {
        let mut scanner = Scanner::new(String::from(" \r\t\n"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn test_string_literal() {
        let mut scanner = Scanner::new(String::from("\"Hello, world!\""));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::String(String::from("Hello, world!")));
        assert_eq!(tokens[1].token_type, TokenType::Eof);
    }

    #[test]
    fn test_identifier() {
        let mut scanner = Scanner::new(String::from("identifier"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Identifier(String::from("identifier")));
        assert_eq!(tokens[1].token_type, TokenType::Eof);
    }

    #[test]
    fn test_keywords() {
        let mut scanner = Scanner::new(String::from("and class else false for fun if nil or print return super this true var while"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 17);
        assert_eq!(tokens[0].token_type, TokenType::And);
        assert_eq!(tokens[1].token_type, TokenType::Class);
        assert_eq!(tokens[2].token_type, TokenType::Else);
        assert_eq!(tokens[3].token_type, TokenType::False);
        assert_eq!(tokens[4].token_type, TokenType::For);
        assert_eq!(tokens[5].token_type, TokenType::Fun);
        assert_eq!(tokens[6].token_type, TokenType::If);
        assert_eq!(tokens[7].token_type, TokenType::Nil);
        assert_eq!(tokens[8].token_type, TokenType::Or);
        assert_eq!(tokens[9].token_type, TokenType::Print);
        assert_eq!(tokens[10].token_type, TokenType::Return);
        assert_eq!(tokens[11].token_type, TokenType::Super);
        assert_eq!(tokens[12].token_type, TokenType::This);
        assert_eq!(tokens[13].token_type, TokenType::True);
        assert_eq!(tokens[14].token_type, TokenType::Var);
        assert_eq!(tokens[15].token_type, TokenType::While);
        assert_eq!(tokens[16].token_type, TokenType::Eof);
    }

    #[test]
    fn test_numbers() {
        let mut scanner = Scanner::new(String::from("1 2 3.15"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_type, TokenType::Number(1.0));
        assert_eq!(tokens[1].token_type, TokenType::Number(2.0));
        assert_eq!(tokens[2].token_type, TokenType::Number(3.15)); // Clippy pls stop complaining about 3.14 and PI...
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_comments() {
        let mut scanner = Scanner::new(String::from("// This is a comment\n// This is another comment"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn test_block_comment() {
        let mut scanner = Scanner::new(String::from("/* This is a \n block comment */"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn small_lox_program() {
        let mut scanner = Scanner::new(String::from("var a = 1;"));
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token_type, TokenType::Var);
        assert_eq!(tokens[1].token_type, TokenType::Identifier(String::from("a")));
        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[3].token_type, TokenType::Number(1.0));
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::Eof);
        assert!(!*rlox::HAD_ERROR.lock().unwrap());
    }

    #[test]
    fn test_error() {
        let mut scanner = Scanner::new(String::from("/* This is a \n unfinished block comment"));
        scanner.scan_tokens();
        assert!(*rlox::HAD_ERROR.lock().unwrap());
    }
}