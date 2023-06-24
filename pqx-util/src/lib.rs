//! file: lib.rs
//! author: Jacob Xie
//! date: 2023/06/04 09:57:46 Sunday
//! brief:

// ================================================================================================
// private macros
// ================================================================================================

#[macro_use]
pub(crate) mod helpers {

    macro_rules! impl_simple_get {
        ($method_name:ident, $path:expr) => {
            pub async fn $method_name(&self) -> crate::error::PqxUtilResult<::serde_json::Value> {
                self.client.get::<_, ::serde_json::Value>($path).await
            }
        };
    }

    macro_rules! impl_get_with_vhost {
        ($method_name:ident, $path:expr) => {
            ::paste::paste! {
                pub async fn [<$method_name _with_vhost>](
                    &self,
                    vhost: &str,
                ) -> crate::error::PqxUtilResult<::serde_json::Value> {
                    let p = format!("{}/{}", $path, vhost);
                    self.client.get::<_, ::serde_json::Value>(p).await
                }
            }
        };
    }
}

// ================================================================================================
// lib
// ================================================================================================

pub mod db;
pub mod error;
pub mod log;
pub mod misc;
pub mod mq;

pub use db::*;
pub use error::*;
pub use log::*;
pub use misc::*;
pub use mq::*;

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

#[macro_export]
macro_rules! now {
    () => {
        ::chrono::Local::now().to_rfc3339()
    };
}
