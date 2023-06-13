//! file: logging.rs
//! author: Jacob Xie
//! date: 2023/06/13 08:14:06 Tuesday
//! brief:

use std::path::Path;

use tracing::{debug, error, info, instrument, warn};

use super::now;

#[instrument]
pub async fn logging_info(s: String) {
    info!("{} {}", now!(), s);
}

#[instrument]
pub async fn logging_error(s: String) {
    error!("{} {}", now!(), s);
}

#[instrument]
pub async fn logging_debug(s: String) {
    debug!("{} {}", now!(), s);
}

#[instrument]
pub async fn logging_warn(s: String) {
    warn!("{} {}", now!(), s);
}

pub fn logging_init(dir: impl AsRef<Path>, filename_prefix: impl AsRef<Path>) {
    let file_appender = tracing_appender::rolling::daily(dir, filename_prefix);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
}
