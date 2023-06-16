//! file: subscriber.rs
//! author: Jacob Xie
//! date: 2023/06/16 00:52:21 Friday
//! brief:

use std::sync::Arc;

use pqx::ec::util::*;
use pqx::error::PqxResult;
use pqx::mq::{MqClient, MqConn, Subscriber};
use pqx::pqx_util::{logging_init, now, read_yaml, PersistClient, PersistConn};
use pqx_app::execution::Executor;
use pqx_app::persistence::MessagePersistent;
use serde::Deserialize;
use tracing::{error, info, instrument};

// ================================================================================================
// Const
// ================================================================================================

const LOGGING_DIR: &str = ".";
const FILENAME_PREFIX: &str = "pqx_subscriber";
const QUE: &str = "pqx.que.1";

// ================================================================================================
// Helper
// ================================================================================================

// PANIC if file not found!
fn get_conn_yaml_path(filename: &str) -> std::path::PathBuf {
    join_dir(current_dir().unwrap(), filename).unwrap()
}

#[instrument]
pub async fn logging_info(s: String) -> PqxResult<()> {
    info!("{} {}", now!(), s);

    Ok(())
}

#[instrument]
pub async fn logging_error(s: String) -> PqxResult<()> {
    error!("{} {}", now!(), s);

    Ok(())
}

// ================================================================================================
// Cfg
// ================================================================================================

#[derive(Deserialize)]
struct Config {
    mq: MqConn,
    db: PersistConn,
}

// ================================================================================================
// Main
// ================================================================================================

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    logging_init(LOGGING_DIR, FILENAME_PREFIX);

    // read config
    let config_path = get_conn_yaml_path("config.yml");
    let config: Config = read_yaml(config_path.to_str().unwrap()).unwrap();

    // setup mq
    let mut mq = MqClient::new();
    mq.connect(config.mq).await.unwrap();
    mq.open_channel(None).await.unwrap();

    // setup db
    let mut ps = PersistClient::new(config.db);
    ps.connect().await.unwrap();
    let mp = MessagePersistent::new(ps.db.unwrap());

    // setup consumer
    let mut consumer = Executor::new(mp);
    consumer
        .exec_mut()
        .register_stdout_fn(Arc::new(logging_info))
        .register_stderr_fn(Arc::new(logging_error));

    let chan = mq.channel().unwrap();
    let mut subscriber = Subscriber::new(chan, consumer);
    subscriber.set_consumer_prefetch(0, 1, false).await.unwrap();

    subscriber.consume(QUE).await.unwrap();

    subscriber.soft_fail_block().await;
}
