//! file: subscriber.rs
//! author: Jacob Xie
//! date: 2023/06/16 00:52:21 Friday
//! brief:

use std::sync::Arc;

use pqx::ec::util::*;
use pqx::error::PqxResult;
use pqx::mq::{MqClient, Subscriber};
use pqx::pqx_util::{logging_init, now, PersistClient};
use pqx_app::execution::Executor;
use pqx_app::persistence::MessagePersistent;
use tracing::{error, info, instrument};

const LOGGING_DIR: &str = ".";
const FILENAME_PREFIX: &str = "pqx_subscriber";
const QUE: &str = "pqx.que.1";

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
// Main
// ================================================================================================

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    logging_init(LOGGING_DIR, FILENAME_PREFIX);

    // mq
    let pth = get_conn_yaml_path("mq.yml");
    let mut mq = MqClient::new();
    mq.connect_by_yaml(pth.to_str().unwrap()).await.unwrap();
    mq.open_channel(None).await.unwrap();

    // db
    let pth = get_conn_yaml_path("db.yml");
    let mut ps = PersistClient::new_by_yaml(pth.to_str().unwrap()).unwrap();
    ps.connect().await.unwrap();
    let mp = MessagePersistent::new(ps.db.unwrap());

    // consumer
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
