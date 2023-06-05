//! file: error.rs
//! author: Jacob Xie
//! date: 2023/06/04 10:17:03 Sunday
//! brief:

use thiserror::Error;

pub type MqApiResult<T> = Result<T, MqApiError>;

#[derive(Debug, Error)]
pub enum MqApiError {
    #[error(transparent)]
    Reqwest(reqwest::Error),

    #[error(transparent)]
    Util(pqx_util::PqxUtilError),

    #[error("{0}")]
    Custom(&'static str),
}

impl MqApiError {
    pub fn custom(e: &'static str) -> Self {
        Self::Custom(e)
    }
}

impl From<reqwest::Error> for MqApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<pqx_util::PqxUtilError> for MqApiError {
    fn from(e: pqx_util::PqxUtilError) -> Self {
        Self::Util(e)
    }
}

impl From<&'static str> for MqApiError {
    fn from(e: &'static str) -> Self {
        Self::Custom(e)
    }
}
