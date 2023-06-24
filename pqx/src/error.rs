//! file: error.rs
//! author: Jacob Xie
//! date: 2023/05/22 21:26:43 Monday
//! brief:

use pqx_util::impl_from_error;
use thiserror::Error;

pub type PqxResult<T> = Result<T, PqxError>;

#[derive(Error, Debug)]
pub enum PqxError {
    #[error(transparent)]
    StdIO(std::io::Error),

    #[error(transparent)]
    NumTryFrom(std::num::TryFromIntError),

    #[error(transparent)]
    RbMQ(amqprs::error::Error),

    #[error(transparent)]
    Serde(serde_json::Error),

    #[error(transparent)]
    Util(pqx_util::PqxUtilError),

    #[error("{0}")]
    Custom(&'static str),
}

impl PqxError {
    pub fn custom(s: &'static str) -> Self {
        Self::Custom(s)
    }
}

impl_from_error!(std::io::Error, PqxError, StdIO);
impl_from_error!(std::num::TryFromIntError, PqxError, NumTryFrom);
impl_from_error!(amqprs::error::Error, PqxError, RbMQ);
impl_from_error!(serde_json::Error, PqxError, Serde);
impl_from_error!(pqx_util::PqxUtilError, PqxError, Util);

impl From<&'static str> for PqxError {
    fn from(e: &'static str) -> Self {
        PqxError::Custom(e)
    }
}

#[macro_export]
macro_rules! pqx_custom_err {
    ($e: expr) => {
        ::pqx::error::PqxError::custom($e)
    };
}
