//! file: lib.rs
//! author: Jacob Xie
//! date: 2023/06/04 09:57:46 Sunday
//! brief:

pub mod cfg;
pub mod error;
pub mod logging;

pub use cfg::*;
pub use error::*;
pub use logging::*;

// ================================================================================================
// private macros
// ================================================================================================

macro_rules! now {
    () => {
        ::chrono::Local::now().to_rfc3339()
    };
}

pub(crate) use now;
