//! file: error.rs
//! author: Jacob Xie
//! date: 2023/05/22 21:26:43 Monday
//! brief:

use thiserror::Error;

pub type PqxResult<T> = Result<T, PqxError>;

#[derive(Error, Debug)]
pub enum PqxError {
    #[error(transparent)]
    StdIO(std::io::Error),

    #[error("{0}")]
    Custom(&'static str),
}

impl PqxError {
    pub fn custom(s: &'static str) -> Self {
        Self::Custom(s)
    }
}

impl From<std::io::Error> for PqxError {
    fn from(e: std::io::Error) -> Self {
        PqxError::StdIO(e)
    }
}

impl From<&'static str> for PqxError {
    fn from(e: &'static str) -> Self {
        PqxError::Custom(e)
    }
}
