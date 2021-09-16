//! The error types of typemake.

use crate::interpreter::InterpreterError;
use nom::{Err, Needed};
use thiserror::Error;

/// An alias of `std::result::Result` with `TypemakeError` as error type.
pub type TypemakeResult<T> = Result<T, TypemakeError>;

/// The main error type of typemake, wrapping all error types that may occur.
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum TypemakeError {
    #[error("Could not parse typefile.")]
    /// An error that occurred in the typefile parser.
    ParserError(String),

    #[error("I/O error.")]
    /// An I/O error.
    IoError(#[from] ::std::io::Error),

    #[error("Error while interpreting script.")]
    /// An error that occurred in the script interpreter.
    InterpreterError(#[from] InterpreterError),

    #[error("An error occurred.")]
    /// An error that does not fit into the other categories.
    GeneralError(String),
}

impl From<String> for TypemakeError {
    fn from(error: String) -> Self {
        Self::GeneralError(error)
    }
}

impl<ErrorType: ToString> From<nom::Err<ErrorType>> for TypemakeError {
    fn from(err: Err<ErrorType>) -> Self {
        match err {
            Err::Incomplete(needed) => match needed {
                Needed::Unknown => {
                    Self::ParserError("missing an unknown number of characters".to_owned())
                }
                Needed::Size(size) => Self::ParserError(format!("missing {} characters", size)),
            },
            Err::Error(e) => {
                Self::ParserError(format!("parser had a recoverable error: {}", e.to_string()))
            }
            Err::Failure(e) => Self::ParserError(format!(
                "parser had an unrecoverable error: {}",
                e.to_string()
            )),
        }
    }
}
