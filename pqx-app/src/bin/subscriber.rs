//! file: subscriber.rs
//! author: Jacob Xie
//! date: 2023/06/16 00:52:21 Friday
//! brief:

use std::sync::Arc;

use clap::Parser;
use pqx::ec::util::*;
use pqx::error::PqxResult;
use pqx::mq::{MqClient, Subscriber};
use pqx::pqx_util::{logging_init, now, read_yaml, PersistClient};
use pqx_app::cfg::ConnectionsConfig;
use pqx_app::execution::Executor;
use pqx_app::persistence::MessagePersistent;
use tracing::{error, info, instrument};

// ================================================================================================
// Const
// ================================================================================================

const LOGGING_DIR: &str = ".";
const FILENAME_PREFIX: &str = "pqx_subscriber";
const CONN_CONFIG: &str = "conn.yml";

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
// Cfg & Args
// ================================================================================================

#[derive(Debug, Parser)]
struct Args {
    que: String,
}

// ================================================================================================
// Main
// ================================================================================================

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    logging_init(LOGGING_DIR, FILENAME_PREFIX);

    // read config
    let config_path = get_conn_yaml_path(CONN_CONFIG);
    let config: ConnectionsConfig = read_yaml(config_path.to_str().unwrap()).unwrap();

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

    // setup subscriber
    let chan = mq.channel().unwrap();
    let mut subscriber = Subscriber::new(chan, consumer);
    subscriber.set_consumer_prefetch(0, 1, false).await.unwrap();

    // start consume
    subscriber.consume(&args.que).await.unwrap();

    // block until fail
    subscriber.soft_fail_block().await;
}
