//! file: error.rs
//! author: Jacob Xie
//! date: 2023/06/04 10:01:03 Sunday
//! brief:

use thiserror::Error;

pub type PqxUtilResult<T> = Result<T, PqxUtilError>;

#[derive(Debug, Error)]
pub enum PqxUtilError {
    #[error(transparent)]
    StdIO(std::io::Error),

    #[error(transparent)]
    SerdeJson(serde_json::Error),

    #[error(transparent)]
    SerdeYaml(serde_yaml::Error),

    #[error(transparent)]
    SeaOrm(sea_orm::error::DbErr),

    #[error("{0}")]
    Custom(&'static str),
}

impl From<std::io::Error> for PqxUtilError {
    fn from(e: std::io::Error) -> Self {
        PqxUtilError::StdIO(e)
    }
}

impl From<serde_json::Error> for PqxUtilError {
    fn from(e: serde_json::Error) -> Self {
        PqxUtilError::SerdeJson(e)
    }
}

impl From<serde_yaml::Error> for PqxUtilError {
    fn from(e: serde_yaml::Error) -> Self {
        PqxUtilError::SerdeYaml(e)
    }
}

impl From<sea_orm::error::DbErr> for PqxUtilError {
    fn from(e: sea_orm::error::DbErr) -> Self {
        PqxUtilError::SeaOrm(e)
    }
}

impl From<&'static str> for PqxUtilError {
    fn from(e: &'static str) -> Self {
        Self::Custom(e)
    }
}
