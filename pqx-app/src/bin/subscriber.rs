//! file: subscriber.rs
//! author: Jacob Xie
//! date: 2023/06/16 00:52:21 Friday
//! brief:

use std::sync::Arc;

use clap::Parser;
use pqx::error::PqxResult;
use pqx::mq::{MqClient, Subscriber};
use pqx::pqx_util::*;
use pqx_app::cfg::{ConnectionsConfig, InitiationsConfig};
use pqx_app::execution::Executor;
use pqx_app::persistence::MessagePersistent;
use tracing::{error, info, instrument};

// ================================================================================================
// Const
// ================================================================================================

const LOGGING_DIR: &str = ".";
const FILENAME_PREFIX: &str = "pqx_subscriber";
const CONN_CONFIG: &str = "conn.yml";
const INIT_CONFIG: &str = "init.yml";

// ================================================================================================
// Helper
// ================================================================================================

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

    let _guard = logging_file_init(LOGGING_DIR, FILENAME_PREFIX).unwrap();

    // read connection config
    let config_path = get_cur_dir_file(CONN_CONFIG).unwrap();
    let conn_config: ConnectionsConfig = read_yaml(config_path.to_str().unwrap()).unwrap();

    // read setup config
    let config_path = get_cur_dir_file(INIT_CONFIG).unwrap();
    let config_path = config_path.to_string_lossy();
    let init_config: InitiationsConfig = read_yaml(config_path).unwrap();

    // setup mq
    let mut mq = MqClient::new();
    mq.connect(conn_config.mq).await.unwrap();
    mq.open_channel(None).await.unwrap();

    // setup db
    let mut ps = PersistClient::new(conn_config.db);
    ps.with_sqlx_logging(false).connect().await.unwrap();
    let mp = MessagePersistent::new(ps.db.unwrap());

    // setup consumer
    let mut consumer = Executor::new(init_config.delayed_exchange, mp);
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
