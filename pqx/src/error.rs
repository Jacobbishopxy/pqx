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

    #[error(transparent)]
    RbMQ(amqprs::error::Error),

    #[error(transparent)]
    Serde(serde_json::Error),

    #[error(transparent)]
    SerdeYaml(serde_yaml::Error),

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

impl From<amqprs::error::Error> for PqxError {
    fn from(e: amqprs::error::Error) -> Self {
        PqxError::RbMQ(e)
    }
}

impl From<serde_json::Error> for PqxError {
    fn from(e: serde_json::Error) -> Self {
        PqxError::Serde(e)
    }
}

impl From<serde_yaml::Error> for PqxError {
    fn from(e: serde_yaml::Error) -> Self {
        PqxError::SerdeYaml(e)
    }
}
