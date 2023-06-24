//! file: logging.rs
//! author: Jacob Xie
//! date: 2023/06/13 08:14:06 Tuesday
//! brief:

use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::prelude::*;

use crate::{PqxUtilError, PqxUtilResult};

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

pub fn logging_file_init(
    dir: &str,
    prefix: &str,
) -> PqxUtilResult<tracing_appender::non_blocking::WorkerGuard> {
    let file_appender = ::tracing_appender::rolling::daily(dir, prefix);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let file_log = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking.with_max_level(tracing::Level::INFO))
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(file_log)
        .try_init()
        .map_err(|_| PqxUtilError::from("tracing_subscriber failed"))?;

    Ok(guard)
}

pub fn logging_init(
    dir: &str,
    prefix: &str,
) -> PqxUtilResult<tracing_appender::non_blocking::WorkerGuard> {
    let stdout_log = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout.with_max_level(tracing::Level::INFO))
        .with_ansi(true)
        .pretty();

    let file_appender = tracing_appender::rolling::daily(dir, prefix);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let file_log = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking.with_max_level(tracing::Level::INFO))
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(file_log)
        .with(stdout_log)
        .try_init()
        .map_err(|_| PqxUtilError::from("tracing_subscriber failed"))?;

    Ok(guard)
}
