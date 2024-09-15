use std::fmt::Display;

use crate::utils::Span;

/// T: Tokenizer, P: Parser
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An error that occurs when an octal, binary or hexadecimal number provided is incomplete - `0x` `0b` `0o`
    TNumberExpected { location: usize },
    /// Invalid octal numbers; Octal numbers with digits 8, or 9
    TInvalidOctal { span: Span },
    /// Invalid binary numbers; Binary numbers with digits 2 to 9
    TInvalidBinary { span: Span },
    /// An error that occurs when an invalid character is encountered
    TInvalidCharacter { location: usize },
    /// End of file
    TEOF,
    /// Invalid expression / statement
    PInvalidStatement {
        message: String,
        span: Span,
    },

    /// Any parser error (Also includes one error generated during bytecode generation)
    PError {
        message: String,
        span: Span,
    },

    /// An internal, or unexpected error
    PInternalError {
        message: String,
        span: Span,
    },

    /// An empty file
    NoResult
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::TNumberExpected { location } => format!("[TOKENIZER ERROR]: Expected number @ {location}"),

            Self::TInvalidCharacter { location } => format!("[TOKENIZER ERROR]: Found invalid character @ {location}"),

            Self::TInvalidBinary { span } => format!("[TOKENIZER ERROR]: Invalid Binary number @ {span}"),

            Self::TInvalidOctal { span } => format!("[TOKENIZER ERROR]: Invalid Octal number @ {span}"),

            // Usually when the file is empty and an early EOF has been produced
            Self::NoResult => format!(""),

            Self::TEOF => format!("End of file reached. No new tokens can be generated"),

            Self::PError { message, span } | Self::PInvalidStatement { message, span } => {
                format!("[PARSE ERROR] {span}: {message}")
            },

            Self::PInternalError { message, span: _ } => format!("[INTERNAL PARSE ERROR]: {message}"),
        };
        write!(f, "{string}")
    }
}