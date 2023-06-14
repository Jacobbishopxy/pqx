//! file: error.rs
//! author: Jacob Xie
//! date: 2023/06/04 10:01:03 Sunday
//! brief:

use thiserror::Error;

use super::impl_from_error;

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

impl_from_error!(std::io::Error, PqxUtilError, StdIO);
impl_from_error!(serde_json::Error, PqxUtilError, SerdeJson);
impl_from_error!(serde_yaml::Error, PqxUtilError, SerdeYaml);
impl_from_error!(sea_orm::error::DbErr, PqxUtilError, SeaOrm);

impl From<&'static str> for PqxUtilError {
    fn from(e: &'static str) -> Self {
        Self::Custom(e)
    }
}
