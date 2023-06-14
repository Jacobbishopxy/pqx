//! file: lib.rs
//! author: Jacob Xie
//! date: 2023/06/04 09:57:46 Sunday
//! brief:

pub mod cfg;
pub mod db;
pub mod error;
pub mod logging;

pub use cfg::*;
pub use db::*;
pub use error::*;
pub use logging::*;

// ================================================================================================
// public macros
// ================================================================================================

#[macro_export]
macro_rules! impl_from_error {
    ($oe:path, $te:ident, $tp:ident) => {
        impl From<$oe> for $te {
            fn from(e: $oe) -> Self {
                $te::$tp(e)
            }
        }
    };
}

// ================================================================================================
// private macros
// ================================================================================================

macro_rules! now {
    () => {
        ::chrono::Local::now().to_rfc3339()
    };
}

pub(crate) use now;
