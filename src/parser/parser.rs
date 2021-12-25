// use crate::expr::{ExpressionStmt, Grouping, Literal, Print, Stmt, Unary};
use crate::expr::*;
use crate::{
    expr::{Binary, Expr},
    tokens::TokenType,
    utils::errors::{ParserError, RoxyError},
    Token,
};
use crate::{RoxyType, TryConversion};

// LANGUAGE GRAMMAR
// program        → statement* EOF ;
// statement      → exprStmt
//                | printStmt ;
// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;
// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

//, -> C
//int a=2 , c=3;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub errors: Vec<RoxyError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        return Self {
            tokens,
            current: 0,
            errors: vec![],
        };
    }

    pub fn synchronize(&mut self, token: &Token) -> Result<(), RoxyError> {
        self.advance(token)?;

        while !self.is_at_end() {
            match self.previous() {
                Some(token) => {
                    if token.token_type == TokenType::Semicolon {
                        return Ok(());
                    }

                    match self.peek() {
                        Some(token) => match token.token_type {
                            TokenType::Class => todo!(),
                            TokenType::Fun => todo!(),
                            TokenType::Var => todo!(),
                            TokenType::For => todo!(),
                            TokenType::If => todo!(),
                            TokenType::While => todo!(),
                            TokenType::Print => todo!(),
                            TokenType::Return => todo!(),
                            _ => (),
                        },
                        None => {
                            return Err(RoxyError::ParserError(ParserError::InvalidTokenAccess(
                                token.to_owned(),
                            )));
                        }
                    }
                }
                None => {
                    return Err(RoxyError::ParserError(ParserError::InvalidTokenAccess(
                        token.to_owned(),
                    )));
                }
            }

            self.advance(token)?;
        }

        Ok(())
    }

    // This has 3 cases:
    // - Ok is returned containing Some(expr), that means parsing is successful
    // - Ok is returned containing None, that means parsing is not successful but no critical errors
    // were reported so check errors field of parser, to get a list of parsing errors
    // - Error is returned meaning there is some critical error
    pub fn parse(&mut self) -> Result<Option<Vec<Stmt>>, RoxyError> {
        let mut stmts: Vec<Stmt> = vec![];
        // println!("Here: {:?}", self.is_at_end());
        while !self.is_at_end() {
            // println!("{:?}", self.is_at_end());
            match self.statement() {
                Ok(stmt) => {
                    stmts.push(stmt);
                }
                Err(err) => {
                    // println!("ParserErr: {:?}", err);
                    match err {
                        RoxyError::ParserError(ref parser_error) => match parser_error {
                            ParserError::InvalidPeek => return Err(err),
                            _ => self.errors.push(err),
                        },
                        _ => unreachable!(),
                    };

                    return Ok(None);
                }
            }
        }

        Ok(Some(stmts))

        //match self.expression() {
        //    Ok(expr) => {
        //        (soem)
        //        //Ok(Some(expr))
        //    },
        //    Err(err) => {
        //        match err {
        //            RoxyError::ParserError(ref parser_error) => match parser_error {
        //                ParserError::InvalidPeek => return Err(err),
        //                _ => self.errors.push(err),
        //            },
        //            _ => unreachable!(),
        //        };

        //        return Ok(None);
        //    }
        //}
    }

    fn statement(&mut self) -> Result<Stmt, RoxyError> {
        let (_, matched) = self.does_any_token_type_match(&vec![TokenType::Print])?;
        if matched {
            return self.print_stmt();
        }

        return self.expr_stmt();
    }

    fn print_stmt(&mut self) -> Result<Stmt, RoxyError> {
        let (last_visited_token, expr) = self.expression()?;
        self.consume(
            &TokenType::Semicolon,
            RoxyError::ParserError(ParserError::ExpectedSemicolon(last_visited_token.clone())),
        )?;
        return Ok(Stmt::Print(Print { expression: expr }));
    }

    fn expr_stmt(&mut self) -> Result<Stmt, RoxyError> {
        let (last_visited_token, expr) = self.expression()?;
        self.consume(
            &TokenType::Semicolon,
            RoxyError::ParserError(ParserError::ExpectedSemicolon(last_visited_token.clone())),
        )?;
        return Ok(Stmt::Expression(ExpressionStmt { expression: expr }));
    }

    pub fn left_recursive_parsing<F>(
        &mut self,
        token_types: &Vec<TokenType>,
        rule_fn: F,
        // rule_fn: &mut dyn FnMut() -> Result<Expr, LoxError>,
    ) -> Result<(Token, Expr), RoxyError>
    where
        F: Fn(&mut Parser) -> Result<(Token, Expr), RoxyError>,
    {
        let mut last_visited_token: Token;
        let (visited_token, mut expr) = rule_fn(self)?;
        last_visited_token = visited_token;

        loop {
            let (visited_token, matched) = self.does_any_token_type_match(&token_types)?;
            if matched {
                break;
            }

            last_visited_token = visited_token;

            match self.previous() {
                Some(operator) => {
                    let operator = operator.to_owned();

                    let (visited_token, right) = rule_fn(self)?;

                    last_visited_token = visited_token;

                    expr = Expr::Binary(Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    })
                }
                None => {
                    return Err(RoxyError::ParserError(ParserError::InvalidTokenAccess(
                        last_visited_token.to_owned(),
                    )));
                }
            }
        }

        Ok((last_visited_token, expr))
    }

    pub fn expression(&mut self) -> Result<(Token, Expr), RoxyError> {
        return self.equality();
    }

    pub fn equality(&mut self) -> Result<(Token, Expr), RoxyError> {
        self.left_recursive_parsing(
            &vec![TokenType::BangEqual, TokenType::EqualEqual],
            Parser::comparison,
        )
    }

    pub fn comparison(&mut self) -> Result<(Token, Expr), RoxyError> {
        self.left_recursive_parsing(
            &vec![
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
            Parser::term,
        )
    }

    pub fn term(&mut self) -> Result<(Token, Expr), RoxyError> {
        self.left_recursive_parsing(&vec![TokenType::Minus, TokenType::Plus], Parser::factor)
    }

    pub fn factor(&mut self) -> Result<(Token, Expr), RoxyError> {
        self.left_recursive_parsing(&vec![TokenType::Slash, TokenType::Star], Parser::unary)
    }

    pub fn unary(&mut self) -> Result<(Token, Expr), RoxyError> {
        let (token, matched) =
            self.does_any_token_type_match(&vec![TokenType::Bang, TokenType::Minus])?;

        if matched {
            match self.previous() {
                Some(operator) => {
                    let operator = operator.to_owned();
                    let (last_visited_token, right) = self.unary()?;
                    return Ok((
                        last_visited_token,
                        Expr::Unary(Unary {
                            operator,
                            right: Box::new(right),
                        }),
                    ));
                }
                None => {
                    return Err(RoxyError::ParserError(ParserError::InvalidTokenAccess(
                        token.to_owned(),
                    )));
                }
            }
        }

        return self.primary();
    }

    pub fn primary(&mut self) -> Result<(Token, Expr), RoxyError> {
        if let (token, Some(expr)) =
            self.match_token_types_and_create_literal(&vec![TokenType::False])?
        {
            return Ok((token, expr));
        }

        if let (token, Some(expr)) =
            self.match_token_types_and_create_literal(&vec![TokenType::True])?
        {
            return Ok((token, expr));
        }

        if let (token, Some(expr)) =
            self.match_token_types_and_create_literal(&vec![TokenType::Nil])?
        {
            return Ok((token, expr));
        }

        if let (token, Some(expr)) =
            self.match_token_types_and_create_literal(&vec![TokenType::Number, TokenType::String])?
        {
            return Ok((token, expr));
        }

        println!("here");
        let (token, matched) = self.does_any_token_type_match(&vec![TokenType::LeftParen])?;
        if matched {
            println!("something");
            let (_, expr) = self.expression()?;
            let last_visited_token = self.consume(
                &TokenType::RightParen,
                RoxyError::ParserError(ParserError::ExpectedRightParen(token.to_owned())),
            )?;
            return Ok((
                last_visited_token,
                Expr::Grouping(Grouping {
                    expr: Box::new(expr),
                }),
            ));
        }

        return Err(RoxyError::ParserError(ParserError::ExpectedExpression(
            token.to_owned(),
        )));
    }

    pub fn consume(&mut self, token_type: &TokenType, err: RoxyError) -> Result<Token, RoxyError> {
        let (token, matched) = self.check(token_type)?;
        if matched {
            return self.advance(&token);
        }

        return Err(err);
    }

    fn match_token_types_and_create_literal(
        &mut self,
        token_types: &Vec<TokenType>,
    ) -> Result<(Token, Option<Expr>), RoxyError> {
        let (token, matched) = self.does_any_token_type_match(token_types)?;
        if matched {
            return match self.previous() {
                Some(prev) => match prev.token_type {
                    TokenType::String => Ok((
                        token,
                        Some(Expr::Literal(Literal {
                            value: RoxyType::String(prev.literal.to_string()),
                        })),
                    )),
                    TokenType::Number => Ok((
                        token.clone(),
                        Some(Expr::Literal(Literal {
                            value: RoxyType::Number(f64::try_conversion(prev.literal, token)?),
                        })),
                    )),
                    TokenType::False => Ok((
                        token,
                        Some(Expr::Literal(Literal {
                            value: RoxyType::Boolean(false),
                        })),
                    )),
                    TokenType::True => Ok((
                        token,
                        Some(Expr::Literal(Literal {
                            value: RoxyType::Boolean(true),
                        })),
                    )),
                    TokenType::Nil => Ok((
                        token,
                        Some(Expr::Literal(Literal {
                            value: RoxyType::NULL,
                        })),
                    )),
                    _ => return Ok((token.clone(), None)),
                },
                None => Err(RoxyError::ParserError(ParserError::InvalidTokenAccess(
                    token.to_owned(),
                ))),
            };
        }

        return Ok((token, None));
    }

    // This should return the token always whether the token matches any of the token type or not,
    // this is because we need this token in upper APIs to do appropriate error reporting
    fn does_any_token_type_match(
        &mut self,
        token_types: &Vec<TokenType>,
    ) -> Result<(Token, bool), RoxyError> {
        let mut last_visited_token: Token;
        let (curr_token, _) = self.check(&token_types[0])?;
        last_visited_token = curr_token;

        let mut i = 0;
        loop {
            if i >= token_types.len() {
                break;
            }

            let token_type = &token_types[i];
            let (token, matched) = self.check(&token_type)?;
            last_visited_token = token;

            if matched {
                last_visited_token = self.advance(&last_visited_token)?;
                return Ok((last_visited_token, true));
            }

            i += 1;
        }

        return Ok((last_visited_token, false));
    }

    // Make this return the token
    fn check(&self, token_type: &TokenType) -> Result<(Token, bool), RoxyError> {
        if let Some(token) = self.peek() {
            let matched = token.token_type == token_type.to_owned();
            return Ok((token, matched));
        }

        return Err(RoxyError::ParserError(ParserError::InvalidPeek));
    }

    fn advance(&mut self, prev_token: &Token) -> Result<Token, RoxyError> {
        if !self.is_at_end() {
            self.current += 1;
        }

        match self.previous() {
            Some(token) => Ok(token.to_owned()),
            None => Err(RoxyError::ParserError(ParserError::InvalidTokenAccess(
                prev_token.to_owned(),
            ))),
        }
    }

    fn previous(&self) -> Option<Token> {
        return Some(self.tokens.get(self.current - 1)?.to_owned());
    }

    fn is_at_end(&self) -> bool {
        if let Some(token) = self.peek() {
            if token.token_type == TokenType::EOF {
                return true;
            }
        }

        false
    }

    fn peek(&self) -> Option<Token> {
        let res = self.tokens.get(self.current)?.to_owned();
        println!("{:?}", res);
        return Some(res);
    }
}
