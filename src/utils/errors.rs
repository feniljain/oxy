use crate::Token;

#[derive(Debug, Clone)]
pub enum RoxyError {
    SyntaxError(CompileTimeError),
    ParserError(ParserError),
    InterpreterError(InterpreterError),
    EnvironmentError(EnvironmentError),
    FileDoesNotExist,
}

impl std::error::Error for RoxyError {}

impl std::fmt::Display for RoxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            RoxyError::SyntaxError(err) => write!(f, "{:?}", err),
            RoxyError::ParserError(err) => write!(f, "{:?}", err),
            RoxyError::InterpreterError(err) => write!(f, "{:?}", err),
            RoxyError::EnvironmentError(err) => write!(f, "{:?}", err),
            RoxyError::FileDoesNotExist => write!(f, "File does not exist"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InterpreterError {
    InvalidUnaryOperator(Token),
    InvalidNumberCast(Token),
    InvalidBooleanCast(Token),
    InvalidStringCast(Token),
    InvalidOperationOnGivenTypes(Token),
    DivideByZeroError(Token),
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InterpreterError::InvalidUnaryOperator(token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Invalid Unary Error",
                    token.line,
                )
            }
            InterpreterError::InvalidNumberCast(token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Invalid number cast Error",
                    token.line,
                )
            }
            InterpreterError::InvalidBooleanCast(token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Invalid boolean cast Error",
                    token.line,
                )
            }
            InterpreterError::InvalidStringCast(token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Invalid string cast Error",
                    token.line,
                )
            }
            InterpreterError::InvalidOperationOnGivenTypes(token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Invalid operation on given types",
                    token.line,
                )
            }
            InterpreterError::DivideByZeroError(token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Divide By Zero Error",
                    token.line,
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileTimeError {
    pub line: usize,
    pub where_in_file: String, //Just because where is a reserved keyword in rust
    pub message: String,
}

impl std::fmt::Display for CompileTimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "[line: {:?}] Error {:?}: {:?}",
            self.line, self.where_in_file, self.message
        )
    }
}

//TODO: Merge errors which need not be seperate
#[derive(Debug, Clone)]
pub enum ParserError {
    InvalidPeek,               // TODO: Think do we need this error?
    InvalidTokenAccess(Token), // Internal parser error, not to be propogated to the users
    InvalidToken(Token),
    ExpectedRightParen(Token),
    ExpectedExpression(Token),
    ExpectedSemicolon(Token),
    ExpectedVariableName(Token),
    ExpectedRightBraceAfterBlock(Token),
    ExpectedLeftBraceAfterKeyword(String, Token),
    ExpectedRightBraceAfterKeyword(String, Token),
    ExpectedSemicolonAfterClauses(Token),
    InvalidAssignmentTarget(Token),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ParserError::InvalidPeek => {
                write!(f, "ParserError[Internal Error]: Invalid peek of token")
            }
            ParserError::InvalidTokenAccess(token) => {
                write!(
                    f,
                    "[line: {:?}] ParserError: Invalid token access: {:?}",
                    token.line, token.lexeme
                )
            }
            ParserError::InvalidToken(token) => write!(
                f,
                "[line: {:?}] ParserError: Invalid token: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedRightParen(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected right paren: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedExpression(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected expression: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedSemicolon(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected semicolon: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedVariableName(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected variable name: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedRightBraceAfterBlock(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected '}}' after block: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedLeftBraceAfterKeyword(keyword, token) => write!(
                f,
                "[line: {:?}] ParserError: Expected '(' after {:?}: {:?}",
                token.line, keyword, token.lexeme
            ),
            ParserError::ExpectedRightBraceAfterKeyword(keyword, token) => write!(
                f,
                "[line: {:?}] ParserError: Expected ')' after {:?}: {:?}",
                token.line, keyword, token.lexeme
            ),
            ParserError::ExpectedSemicolonAfterClauses(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected ';' after clauses: {:?}",
                token.line, token.lexeme
            ),
            ParserError::InvalidAssignmentTarget(token) => write!(
                f,
                "[line: {:?}] ParserError: Invalid  assignment target: {:?}",
                token.line, token.lexeme
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EnvironmentError {
    UndefinedVariable(String),
}

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            EnvironmentError::UndefinedVariable(var_name) => {
                write!(f, "EnvironmentError: Undefined variable: {:?}.", var_name)
            }
        }
    }
}

// impl From<ParserError> for SyntaxError {
//     fn from(parser_error: ParserError) -> Self {
//         match parser_error {
//             ParserError::InvalidTokenAccess(ref token) => SyntaxError {
//                 line: token.line,
//                 where_in_file: where_in_file(&token),
//                 message: parser_error.clone().to_string(),
//             },
//             ParserError::InvalidToken(ref token) => SyntaxError {
//                 line: token.line,
//                 where_in_file: where_in_file(&token),
//                 message: parser_error.to_string(),
//             },
//             ParserError::ExpectedRightParen(ref token) => SyntaxError {
//                 line: token.line,
//                 where_in_file: where_in_file(&token),
//                 message: parser_error.to_string(),
//             },
//             ParserError::ExpectedExpression(ref token) => SyntaxError {
//                 line: token.line,
//                 where_in_file: where_in_file(&token),
//                 message: parser_error.to_string(),
//             },
//             ParserError::ExpectedSemicolon(ref token) => SyntaxError {
//                 line: token.line,
//                 where_in_file: where_in_file(&token),
//                 message: parser_error.to_string(),
//             },
//             ParserError::InvalidPeek => SyntaxError {
//                 line: 999999,
//                 where_in_file: String::from("where_in_file"),
//                 message: parser_error.to_string(),
//             },
//         }
//     }
// }

// fn where_in_file(token: &Token) -> String {
//     if token.token_type == TokenType::EOF {
//         return format!("at end");
//     }

//     return format!("at '{:?}", token.lexeme);
// }
