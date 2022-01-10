use std::collections::HashMap;

use crate::utils::errors::{CompileTimeError, RoxyError};
use crate::Token;
use crate::{tokens::TokenType, RoxyType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
}

//TODO: Fix multiline comments
impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0,
            keywords: construct_keywords(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, RoxyError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: String::new(),
            literal: RoxyType::NULL,
            line: self.line,
        });

        return Ok(&self.tokens);
    }

    pub fn is_at_end(&self) -> bool {
        return self.current > self.source.len() - 1;
    }

    pub fn scan_token(&mut self) -> anyhow::Result<(), RoxyError> {
        if let Some(c) = self.advance() {
            match c {
                '(' => self.add_token(TokenType::LeftParen, None),
                ')' => self.add_token(TokenType::RightParen, None),
                '{' => self.add_token(TokenType::LeftBrace, None),
                '}' => self.add_token(TokenType::RightBrace, None),
                ',' => self.add_token(TokenType::Comma, None),
                '.' => self.add_token(TokenType::Dot, None),
                '-' => self.add_token(TokenType::Minus, None),
                '+' => self.add_token(TokenType::Plus, None),
                ';' => self.add_token(TokenType::Semicolon, None),
                '*' => self.add_token(TokenType::Star, None),
                '=' => {
                    if self.lookahead_one_step('=') {
                        self.add_token(TokenType::EqualEqual, None);
                    } else {
                        self.add_token(TokenType::Equal, None)
                    }
                }
                '<' => {
                    if self.lookahead_one_step('=') {
                        self.add_token(TokenType::LessEqual, None);
                    } else {
                        self.add_token(TokenType::Less, None)
                    }
                }
                '>' => {
                    if self.lookahead_one_step('=') {
                        self.add_token(TokenType::GreaterEqual, None);
                    } else {
                        self.add_token(TokenType::Greater, None)
                    }
                }
                '!' => {
                    if self.lookahead_one_step('=') {
                        self.add_token(TokenType::BangEqual, None);
                    } else {
                        self.add_token(TokenType::Bang, None)
                    }
                }
                '/' => {
                    if self.lookahead_one_step('/') {
                        while let Some(ch) = self.peek() {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.lookahead_one_step('*') {
                        let mut slash_star_encountered = 1;
                        while let Some(ch) = self.peek() {
                            if ch == '\n' {
                                self.line += 1;
                            }

                            if ch == '/' {
                                if let Some(ch_next) = self.peek_next() {
                                    if ch_next == '*' {
                                        self.advance();
                                        self.advance();
                                        slash_star_encountered += 1;
                                    }
                                    continue;
                                } else {
                                    return Err(RoxyError::SyntaxError(CompileTimeError {
                                        line: self.line,
                                        where_in_file: String::new(),
                                        message: String::from("Unterminated multiline comment"),
                                    }));
                                }
                            }

                            if ch == '*' {
                                if let Some(ch_next) = self.peek_next() {
                                    if ch_next == '/' {
                                        self.advance();
                                        self.advance();
                                        slash_star_encountered -= 1;
                                        if slash_star_encountered == 0 {
                                            break;
                                        }
                                    }
                                    continue;
                                } else {
                                    return Err(RoxyError::SyntaxError(CompileTimeError {
                                        line: self.line,
                                        where_in_file: String::new(),
                                        message: String::from("Unterminated multiline comment"),
                                    }));
                                }
                            }
                            self.advance();
                        }
                    } else {
                        self.add_token(TokenType::Slash, None)
                    }
                }
                ' ' => {}
                '\r' => {}
                '\t' => {}
                '\n' => {
                    self.line += 1;
                }
                '"' => self.string()?,
                _ => {
                    if self.is_digit(c) {
                        self.number();
                    } else if self.is_alpha(c) {
                        self.identifier();
                    } else {
                        return Err(RoxyError::SyntaxError(CompileTimeError {
                            line: self.line,
                            where_in_file: String::new(),
                            message: String::from("Unparsable token"),
                        }));
                    }
                }
            }
            return Ok(());
        }

        Err(RoxyError::SyntaxError(CompileTimeError {
            line: self.line,
            where_in_file: String::new(),
            message: String::from("panic in scanner advancing"),
        }))
    }

    fn identifier(&mut self) {
        while let Some(ch) = self.peek() {
            if !self.is_alpha_numeric(ch) {
                break;
            }
            self.advance();
        }

        let chars: Vec<char> = self.source.chars().collect();
        let value: String = chars[self.start..self.current].iter().collect();

        let token_type: TokenType;
        if let Some(kv) = self.keywords.get_key_value(&value) {
            token_type = kv.1.to_owned();
        } else {
            token_type = TokenType::Identifier;
        }

        self.add_token(token_type, Some(RoxyType::String(value)));
    }

    fn is_alpha_numeric(&self, ch: char) -> bool {
        return self.is_alpha(ch) || self.is_digit(ch);
    }

    fn number(&mut self) {
        while let Some(ch) = self.peek() {
            if !self.is_digit(ch) {
                break;
            }
            self.advance();
        }

        if let Some(ch) = self.peek_next() {
            if self.peek() == Some('.') && self.is_digit(ch) {
                self.advance();
            }
        }

        while let Some(ch) = self.peek() {
            if !self.is_digit(ch) {
                break;
            }
            self.advance();
        }

        let chars: Vec<char> = self.source.chars().collect();
        let value: String = chars[self.start..self.current].iter().collect();
        if let Ok(num) = value.parse::<f64>() {
            self.add_token(TokenType::Number, Some(RoxyType::Number(num)));
        }
    }

    fn is_digit(&self, ch: char) -> bool {
        return ch >= '0' && ch <= '9';
    }

    fn is_alpha(&self, ch: char) -> bool {
        return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_';
    }

    fn string(&mut self) -> Result<(), RoxyError> {
        while self.peek() != Some('"') && self.peek() != None {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(RoxyError::SyntaxError(CompileTimeError {
                line: self.line,
                where_in_file: String::new(),
                message: String::from("Unterminated String"),
            }));
        }

        self.advance();

        let chars: Vec<char> = self.source.chars().collect();
        let value: String = chars[self.start + 1..self.current - 1].iter().collect();
        self.add_token(TokenType::String, Some(RoxyType::String(value)));

        Ok(())
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current)
        // if let Some(ch) =  {
        //     return ch;
        // } else {
        //     return '\0';
        // }
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }

    fn lookahead_one_step(&mut self, ch: char) -> bool {
        if let Some(curr_ch) = self.source.chars().nth(self.current) {
            if ch != curr_ch {
                return false;
            }
            self.current += 1;
            return true;
        }

        return false;
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.chars().nth(self.current);
        self.current += 1;
        return ch;
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<RoxyType>) {
        let text = self.source[self.start..self.current].to_string();
        let literal = match literal {
            Some(l) => l,
            None => RoxyType::NULL,
        };
        self.tokens.push(Token {
            token_type,
            lexeme: text,
            literal,
            line: self.line,
        });
    }
}

fn construct_keywords() -> HashMap<String, TokenType> {
    let mut keywords: HashMap<String, TokenType> = HashMap::new();

    keywords.insert(String::from("and"), TokenType::And);
    keywords.insert(String::from("class"), TokenType::Class);
    keywords.insert(String::from("else"), TokenType::Else);
    keywords.insert(String::from("false"), TokenType::False);
    keywords.insert(String::from("for"), TokenType::For);
    keywords.insert(String::from("fun"), TokenType::Fun);
    keywords.insert(String::from("if"), TokenType::If);
    keywords.insert(String::from("nil"), TokenType::Nil);
    keywords.insert(String::from("or"), TokenType::Or);
    keywords.insert(String::from("print"), TokenType::Print);
    keywords.insert(String::from("return"), TokenType::Return);
    keywords.insert(String::from("super"), TokenType::Super);
    keywords.insert(String::from("this"), TokenType::This);
    keywords.insert(String::from("true"), TokenType::True);
    keywords.insert(String::from("var"), TokenType::Var);
    keywords.insert(String::from("while"), TokenType::While);

    return keywords;
}
