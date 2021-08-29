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
