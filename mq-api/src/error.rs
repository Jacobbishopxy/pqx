//! file: error.rs
//! author: Jacob Xie
//! date: 2023/06/04 10:17:03 Sunday
//! brief:

use pqx_util::impl_from_error;
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

impl_from_error!(reqwest::Error, MqApiError, Reqwest);
impl_from_error!(pqx_util::PqxUtilError, MqApiError, Util);

impl From<&'static str> for MqApiError {
    fn from(e: &'static str) -> Self {
        Self::Custom(e)
    }
}
