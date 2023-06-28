//! file: inspector.rs
//! author: Jacob Xie
//! date: 2023/06/26 00:03:35 Monday
//! brief:

use clap::Parser;
use pqx::error::PqxResult;
use pqx::mq::MqClient;
use pqx::pqx_custom_err;
use pqx::pqx_util::*;
use pqx_app::adt::{BindingInfo, ExchangeInfo, QueueInfo};
use pqx_app::cfg::ConnectionsConfig;
use pqx_app::persist::MessagePersistent;
use tracing::info;

// ================================================================================================
// Const
// ================================================================================================

// commands
const INSP: &str = "insp";

// default constants
const LOGGING_DIR: &str = "./logs";
const FILENAME_PREFIX: &str = "pqx_inspector";
const CONN_CONFIG: &str = "conn.yml";

static DEFAULT_EXCHANGE: &[&str] = &[
    "",
    "amq.direct",
    "amq.fanout",
    "amq.headers",
    "amq.match",
    "amq.rabbitmq.trace",
    "amq.topic",
];

// ================================================================================================
// Args
// ================================================================================================

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    option: String,
    config: Option<String>,
}

// ================================================================================================
// Fn
// ================================================================================================

async fn check_tables(client: &PersistClient) -> PqxResult<(bool, bool)> {
    let db = client.db().expect("connection is established");
    let mp = MessagePersistent::new(db.clone());

    mp.check_existence().await
}

async fn check_mq(
    client: &MqApiClient,
) -> PqxResult<(Vec<ExchangeInfo>, Vec<QueueInfo>, Vec<BindingInfo>)> {
    let query = MqQuery::new(&client);

    let res1 = query
        .exchanges_with_vhost(client.vhost())
        .await?
        .as_array()
        .ok_or(pqx_custom_err!("array"))?
        .into_iter()
        .map(ExchangeInfo::try_from)
        .collect::<PqxResult<Vec<_>>>()?
        .into_iter()
        .filter(|e| !DEFAULT_EXCHANGE.contains(&e.name.as_ref()))
        .collect::<Vec<_>>();
    let res2 = query
        .queues_with_vhost(&client.vhost())
        .await?
        .as_array()
        .ok_or(pqx_custom_err!("array"))?
        .into_iter()
        .map(QueueInfo::try_from)
        .collect::<PqxResult<Vec<_>>>()?;
    let res3 = query
        .bindings_with_vhost(&client.vhost())
        .await?
        .as_array()
        .ok_or(pqx_custom_err!("array"))?
        .into_iter()
        .map(BindingInfo::try_from)
        .collect::<PqxResult<Vec<_>>>()?;

    Ok((res1, res2, res3))
}

// ================================================================================================
// Main
// ================================================================================================

/// Options
///
/// 0. cargo run --bin inspector -- -o insp
#[tokio::main]
async fn main() {
    let args = Args::parse();

    let _guard = logging_init(LOGGING_DIR, FILENAME_PREFIX, tracing::Level::INFO).unwrap();

    info!("{} Start inspector... 🫨", now!());

    // read connection config
    let config_path = get_cur_dir_file(CONN_CONFIG).unwrap();
    let config_path = config_path.to_string_lossy();
    let config: ConnectionsConfig = read_yaml(config_path).unwrap();

    // mq client
    let mut mq_client = MqClient::new();
    mq_client.connect(config.mq).await.unwrap();
    mq_client.open_channel(None).await.unwrap();
    // db client
    let mut db_client = PersistClient::new(config.db);
    db_client.with_sqlx_logging(false).connect().await.unwrap();
    // mq-api client
    let api_client = MqApiClient::new(config.mq_api);

    match args.option.as_str() {
        INSP => {
            info!("{} INSP", now!());
            // check tables
            let res = check_tables(&db_client).await.unwrap();
            info!(
                "{} table existence > message_history: {}, message_result: {}",
                now!(),
                res.0,
                res.1
            );

            // check exchanges, queues and bindings exist
            let (res1, res2, res3) = check_mq(&api_client).await.unwrap();
            info!("{} [ExchangeInfo] {:?}", now!(), &res1);
            info!("{} [QueueInfo] {:?}", now!(), &res2);
            info!("{} [BindingInfo] {:?}", now!(), &res3);
        }
        _ => panic!("undefined option"),
    }

    info!("{} End inspector 😎", now!());
}
