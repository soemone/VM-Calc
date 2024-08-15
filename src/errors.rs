use std::fmt::Display;

use crate::utils::Span;

/// T: Tokenizer, P: Parser
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An error that occurs when an octal, binary or hexadecimal number provided is incomplete - `0x` `0b` `0o`
    TNumberExpected,
    /// An error that occurs when an invalid character is encountered
    TInvalidCharacter,
    /// End of file
    TEOF,
    /// Invalid expression / statement
    PInvalidStatement {
        message: String,
        span: Span,
    },

    /// Any parser error
    PError {
        message: String,
        span: Span,
    },

    /// An internal, or unexpected error
    PInternalError {
        message: String,
        span: Span,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::TNumberExpected | Self::TInvalidCharacter => {
                format!("Tokenizer Error {self:?}")
            },

            Self::TEOF => {
                format!("End of file reached. No new tokens can be generated")
            },

            Self::PError { message, span: _ } | Self::PInvalidStatement { message, span: _ } => {
                format!("[PARSE ERROR]: {message}")
            },

            Self::PInternalError { message, span: _ } => {
                format!("[INTERNAL PARSE ERROR]: {message}")
            },
        };
        write!(f, "{string}")
    }
}