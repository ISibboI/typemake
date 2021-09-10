use nom::{Err, Needed};
use thiserror::Error;

pub type TypemakeResult<T> = Result<T, TypemakeError>;

#[derive(Error, Debug)]
pub enum TypemakeError {
    #[error("could not parse typefile")]
    ParseError(String),

    #[error("I/O error")]
    IoError(#[from] ::std::io::Error),
}

impl PartialEq for TypemakeError {
    fn eq(&self, other: &Self) -> bool {
        match self {
            TypemakeError::ParseError(error) => {
                if let TypemakeError::ParseError(other_error) = other {
                    error == other_error
                } else {
                    false
                }
            }
            TypemakeError::IoError(_) => false,
        }
    }
}

impl<ErrorType: ToString> From<nom::Err<ErrorType>> for TypemakeError {
    fn from(err: Err<ErrorType>) -> Self {
        match err {
            Err::Incomplete(needed) => match needed {
                Needed::Unknown => {
                    Self::ParseError("missing an unknown number of characters".to_owned())
                }
                Needed::Size(size) => Self::ParseError(format!("missing {} characters", size)),
            },
            Err::Error(e) => {
                Self::ParseError(format!("parser had a recoverable error: {}", e.to_string()))
            }
            Err::Failure(e) => Self::ParseError(format!(
                "parser had an unrecoverable error: {}",
                e.to_string()
            )),
        }
    }
}
