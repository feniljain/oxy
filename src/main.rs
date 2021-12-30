pub mod callable;
pub mod environment;
pub mod expr;
pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod tokens;
pub mod utils;

use core::fmt::Debug;
use environment::Environment;
use expr::Stmt;
use parser::Parser;
use std::fmt::{self, Formatter};
use tokens::TokenType;

use ctrlc;
use interpreter::Interpreter;
use std::io::Write;
use std::{env::args, fs, process::exit};
use utils::errors::{InterpreterError, RoxyError};

// TODO: Write tests for every component

//TODO: Move all these into a separate file
#[derive(Debug, Clone)]
pub enum RoxyType {
    String(String),
    Number(f64),
    NULL,
    Boolean(bool),
    Object,
    RoxyFunction(RoxyFunction),
    NativeFunction(NativeFunction),
}

//TODO: Implement Debug properly for both functions
#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub params: Vec<Token>,
    pub callable: fn(
        roxy_type: &RoxyType,
        interpreter: &mut Interpreter,
        arguments: Vec<RoxyType>,
        token: Token,
    ) -> Result<RoxyType, RoxyError>,
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("NativeFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish()
    }
}

impl std::fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<{:?}>", self.name)
    }
}

#[derive(Clone)]
pub struct RoxyFunction {
    pub name: String,
    pub arity: usize,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub closure: Box<Environment>,
}

impl Debug for RoxyFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("RoxyFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish()
    }
}

impl std::fmt::Display for RoxyFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<{:?}>", self.name)
    }
}

impl PartialEq for RoxyType {
    fn eq(&self, other: &Self) -> bool {
        //TODO: Confirm if objects case is rightly implemented
        match (self, other) {
            (RoxyType::String(val_left), RoxyType::String(val_right)) => val_left == val_right,
            (RoxyType::Number(val_left), RoxyType::Number(val_right)) => val_left == val_right,
            (RoxyType::NULL, RoxyType::NULL) => true,
            (RoxyType::NULL, RoxyType::Boolean(val)) => &false == val,
            (RoxyType::Boolean(val), RoxyType::NULL) => val == &true,
            (RoxyType::Boolean(val_left), RoxyType::Boolean(val_right)) => val_left == val_right,
            _ => false,
        }
    }
}

impl std::fmt::Display for RoxyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        //TODO: Improve Object part
        //TODO: Improve NativeFunc & RoxyFunction part
        match self {
            RoxyType::String(streeng) => write!(f, "{}", streeng),
            RoxyType::Number(no) => write!(f, "{}", no),
            RoxyType::NULL => write!(f, "NULL"),
            RoxyType::Boolean(boole) => write!(f, "{:?}", boole),
            RoxyType::Object => write!(f, "object"),
            RoxyType::RoxyFunction(_) => write!(f, "RoxyFunction"),
            RoxyType::NativeFunction(_) => write!(f, "NativeFunction"),
        }
    }
}

trait TryConversion<T>: Sized {
    fn try_conversion(value: RoxyType, token: Token) -> Result<T, RoxyError>;
}

impl TryConversion<f64> for f64 {
    fn try_conversion(value: RoxyType, token: Token) -> Result<Self, RoxyError> {
        match value {
            RoxyType::Number(val) => Ok(val),
            _ => Err(RoxyError::InterpreterError(
                InterpreterError::InvalidNumberCast(token),
            )),
        }
    }
}

impl TryConversion<bool> for bool {
    fn try_conversion(value: RoxyType, token: Token) -> Result<Self, RoxyError> {
        match value {
            RoxyType::Boolean(val) => Ok(val),
            RoxyType::NULL => Ok(false),
            RoxyType::String(_) => Ok(true),
            _ => Err(RoxyError::InterpreterError(
                InterpreterError::InvalidBooleanCast(token),
            )),
        }
    }
}

impl TryConversion<String> for String {
    fn try_conversion(value: RoxyType, token: Token) -> Result<Self, RoxyError> {
        match value {
            RoxyType::String(val) => Ok(val),
            _ => Err(RoxyError::InterpreterError(
                InterpreterError::InvalidStringCast(token),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: RoxyType,
    line: usize,
}

impl Token {
    // fn new(token_type: TokenType, lexeme: String, literal: RoxyType, line: usize) -> Self {
    //     Self {
    //         token_type,
    //         lexeme,
    //         literal,
    //         line,
    //     }
    // }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:?} {:?} {:?}",
            self.token_type.to_string().as_str().to_owned(),
            self.lexeme.as_str(),
            self.literal.to_string().as_str(),
        )
    }
}

struct CliHandler {
    had_err: bool,
    had_runtime_err: bool,
}

impl CliHandler {
    fn new() -> Self {
        Self {
            had_err: false,
            had_runtime_err: false,
            // error: None,
        }
    }

    fn run_prompt(&mut self) -> anyhow::Result<()> {
        println!("Running Prompt");
        //TODO: Later check how to convert it into ctrl+d
        ctrlc::set_handler(move || {
            println!("exiting because of press of ctrl+c");
            exit(64);
        })?;

        loop {
            let input = CliHandler::prompt("> ");

            if input == "exit" {
                break;
            } else if input != "" {
                self.run(input)?;
                self.had_err = false;
            }
        }

        Ok(())
    }

    fn run_file(&self, file_name: &str) -> anyhow::Result<()> {
        let contents = fs::read_to_string(file_name)?;
        self.run(contents)?;

        if self.had_err {
            exit(65);
        }

        if self.had_runtime_err {
            exit(70);
        }

        Ok(())
    }

    fn run(&self, contents: String) -> Result<(), RoxyError> {
        let mut scanner = scanner::Scanner::new(contents.clone());
        let tokens = scanner.scan_tokens()?;

        // println!("{:?}", tokens);
        // for token in tokens {
        //     print!("{:?} - ", token.lexeme);
        // }
        // println!("\n=========================");

        //TODO: Enable supoprt for expr execution
        // let mut parser = Parser::new(tokens.clone());
        // match parser.parse_expression() {
        //     Ok(expr_opt) => {
        //         if let Some(expr) = expr_opt {
        //             println!("Parsing expr successful");
        //             let mut interpreter = Interpreter::new();
        //             match interpreter.evaluate(&expr) {
        //                 Ok(expr) => {
        //                     println!("Expr: {:?}", expr);
        //                 }
        //                 Err(err) => {
        //                     println!("Runtime Error: {:?}", err.to_string())
        //                 }
        //             }
        //         } else {
        //             println!("Parsing Expr Unsuccessful");
        //             for parsing_error in parser.errors {
        //                 println!("Expr error: {:?}", parsing_error);
        //             }
        //         }
        //     }
        //     Err(err) => {
        //         println!(
        //             "Parsing Expr Unsuccessful Critical Error: {:?}",
        //             err.to_string()
        //         );
        //     }
        // }

        let mut parser = Parser::new(tokens.clone());
        match parser.parse() {
            Ok(stmts_opt) => {
                match stmts_opt {
                    Some(stmts) => {
                        // println!("Parsing Successful: {:?}", stmts);
                        let mut interpreter = Interpreter::new();
                        for stmt in stmts {
                            interpreter.interpret(stmt)?;
                        }
                    }
                    None => {
                        println!("Parsing Statement Unsuccessful");
                        for parsing_error in parser.errors {
                            println!("{:?}", parsing_error);
                        }
                    }
                }
            }
            Err(err) => {
                println!("Parsing Statement Unsuccessful Critical Error");
                return Err(err);
            }
        }

        Ok(())
    }

    fn prompt(name: &str) -> String {
        let mut line = String::new();
        print!("{}", name);
        std::io::stdout().flush().unwrap();
        std::io::stdin()
            .read_line(&mut line)
            .expect("Error: Could not read a line");

        return line.trim().to_string();
    }
}

fn main() {
    let args_n = args().len() - 1;
    let arguments: Vec<String> = args().collect();

    let mut interpreter = CliHandler::new();

    if args_n > 1 {
        println!("Usage: roxy [script]");
        exit(64);
    } else if args_n == 1 {
        match interpreter.run_file(&arguments[1]) {
            Ok(_) => (),
            // TODO: Confirm if this code is proper for exit with this error
            Err(err) => {
                println!("{:?}", err);
                exit(64);
            }
        }
    } else {
        match interpreter.run_prompt() {
            Ok(_) => (),
            //TODO: Confirm if this code is proper for exit with this error
            Err(err) => {
                println!("{:?}", err);
                exit(64);
            }
        }
    }
}
