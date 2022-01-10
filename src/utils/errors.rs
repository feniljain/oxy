use crate::Token;

#[derive(Debug, Clone)]
pub enum RoxyError {
    SyntaxError(CompileTimeError),
    ParserError(ParserError),
    InterpreterError(InterpreterError),
    EnvironmentError(EnvironmentError),
    ResolutionError(ResolutionError),
    InternalError(InternalError),
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
            RoxyError::ResolutionError(err) => write!(f, "{:?}", err),
            RoxyError::InternalError(err) => write!(f, "{:?}", err),
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
    ExpectednArgsGotmArgs(usize, usize, Token),
    DivideByZeroError(Token),
    CanOnlyCallFunctionsAndClasses(Token),
    OnlyInstancesHaveKeyword(String, Token),
    UndefinedProperty(Token),
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
            InterpreterError::CanOnlyCallFunctionsAndClasses(token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Can only call functions and classes",
                    token.line,
                )
            }
            InterpreterError::ExpectednArgsGotmArgs(n, m, token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Expected {:?} arguments but got {:?} args",
                    token.line, n, m,
                )
            }
            InterpreterError::OnlyInstancesHaveKeyword(keyword, token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Only Instances are expected to have {:?}: {:?}",
                    token.line, keyword, token.lexeme
                )
            }
            InterpreterError::UndefinedProperty(token) => {
                write!(
                    f,
                    "[line: {:?}] InterpreterError: Undefined Property: {:?}",
                    token.line, token.lexeme,
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
    ExpectedLeftParen(Token),
    ExpectedRightParen(Token),
    ExpectedExpression(Token),
    ExpectedSemicolon(Token),
    ExpectedIdentifier(String, String, Token),
    ExpectedVariableName(Token),
    ExpectedParameterName(Token),
    ExpectedRightBraceAfterBlock(Token),
    ExpectedPunctAfterKeyword(String, String, Token),
    ExpectedSemicolonAfterClauses(Token),
    InvalidAssignmentTarget(Token),
    CannotHaveMoreThan255Arguments(Token),
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
            ParserError::ExpectedLeftParen(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected left paren: {:?}",
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
            ParserError::ExpectedIdentifier(kind, kw, token) => write!(
                f,
                "[line: {:?}] ParserError: Expected {:?} {:?}: {:?}",
                token.line, kind, kw, token.lexeme
            ),
            ParserError::ExpectedVariableName(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected variable name: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedParameterName(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected parameter name: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedRightBraceAfterBlock(token) => write!(
                f,
                "[line: {:?}] ParserError: Expected '}}' after block: {:?}",
                token.line, token.lexeme
            ),
            ParserError::ExpectedPunctAfterKeyword(punctuation, keyword, token) => write!(
                f,
                "[line: {:?}] ParserError: Expected {} after {}: {:?}",
                token.line, punctuation, keyword, token.lexeme
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
            ParserError::CannotHaveMoreThan255Arguments(token) => write!(
                f,
                "[line: {:?}] ParserError: Cannot have more than 255 arguments: {:?}",
                token.line, token.lexeme
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EnvironmentError {
    UndefinedVariable(String),
    EnvironmentDoesNotExistAtGivenDistance,
}

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            EnvironmentError::UndefinedVariable(var_name) => {
                write!(f, "EnvironmentError: Undefined variable: {:?}.", var_name)
            }
            EnvironmentError::EnvironmentDoesNotExistAtGivenDistance => {
                write!(
                    f,
                    "EnvironmentError: Environment Does Not Exist At Given Distance"
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolutionError {
    CantReadLocalVariableInItsOwnInitializer(Token),
    InvalidScopeAccess(Token),
    AlreadyAVariableWithThisNameInThisScope(Token),
    CantReturnFromTopLevelCode(Token),
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ResolutionError::CantReadLocalVariableInItsOwnInitializer(token) => {
                write!(
                    f,
                    "[line: {:?}] ResolutionError: Cant read local varaiable in its own initializer: {:?}",
                    token.line, token.lexeme
                )
            }
            ResolutionError::InvalidScopeAccess(token) => {
                write!(
                    f,
                    "[line: {:?}] ResolutionError: Invalid Scope Access: {:?}",
                    token.line, token.lexeme
                )
            }
            ResolutionError::AlreadyAVariableWithThisNameInThisScope(token) => {
                write!(
                    f,
                    "[line: {:?}] ResolutionError: A variable with same name already exists in the scope: {:?}",
                    token.line, token.lexeme
                )
            }
            ResolutionError::CantReturnFromTopLevelCode(token) => {
                write!(
                    f,
                    "[line: {:?}] ResolutionError: Can't return from top level code: {:?}",
                    token.line, token.lexeme
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InternalError {
    TimeConversionError(Token),
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InternalError::TimeConversionError(token) => {
                write!(
                    f,
                    "[line: {:?}] InternalError: Time Conversion Failed: {:?}",
                    token.line, token.lexeme
                )
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
