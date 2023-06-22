//! file: lib.rs
//! author: Jacob Xie
//! date: 2023/06/04 00:09:42 Sunday
//! brief:

// ================================================================================================
// macros
// ================================================================================================

#[macro_use]
pub(crate) mod helpers {

    macro_rules! impl_simple_get {
        ($method_name:ident, $path:expr) => {
            pub async fn $method_name(&self) -> crate::error::MqApiResult<::serde_json::Value> {
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
                ) -> crate::error::MqApiResult<::serde_json::Value> {
                    let p = format!("{}/{}", $path, vhost);
                    self.client.get::<_, ::serde_json::Value>(p).await
                }
            }
        };
    }
}

pub mod client;
pub mod error;
pub mod query;

pub use client::*;
pub use error::*;
pub use query::*;
