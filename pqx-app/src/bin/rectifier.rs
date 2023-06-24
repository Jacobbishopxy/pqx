//! file: rectifier.rs
//! author: Jacob Xie
//! date: 2023/06/24 22:39:46 Saturday
//! brief:

use clap::Parser;
use pqx::error::PqxResult;
use pqx::mq::MqClient;
use pqx::pqx_util::*;
use pqx_app::cfg::{ConnectionsConfig, InitiationsConfig};
use pqx_app::persist::MessagePersistent;
use tracing::info;

// ================================================================================================
// Const
// ================================================================================================

// commands
const DEL_QUE: &str = "del_que";
const DEL_EXG: &str = "del_exg";
const DEL_TBL: &str = "del_tbl";

// default constants
const LOGGING_DIR: &str = "./logs";
const FILENAME_PREFIX: &str = "pqx_rectifier";
const CONN_CONFIG: &str = "conn.yml";
const INIT_CONFIG: &str = "init.yml";

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

async fn delete_queues(client: &MqClient, config: &InitiationsConfig) -> PqxResult<()> {
    for hq in config.header_queues.iter() {
        client.delete_queue(&hq.queue).await?;
    }

    Ok(())
}

async fn delete_exchanges(client: &MqClient, config: &InitiationsConfig) -> PqxResult<()> {
    client.delete_exchange(&config.header_exchange).await?;
    client.delete_exchange(&config.delayed_exchange).await?;
    client.delete_exchange(&config.dead_letter_exchange).await?;

    Ok(())
}

async fn delete_tables(client: &PersistClient) -> PqxResult<()> {
    let db = client.db().expect("connection is established");
    let mp = MessagePersistent::new(db.clone());

    mp.drop_table().await?;

    Ok(())
}

// ================================================================================================
// Main
// ================================================================================================

/// Main
///
/// 0. cargo run --bin rectifier -- -o del_que
/// 1. cargo run --bin rectifier -- -o del_exg
/// 2. cargo run --bin rectifier -- -o del_tbl
#[tokio::main]
async fn main() {
    let args = Args::parse();

    // logger
    let _guard = logging_init(LOGGING_DIR, FILENAME_PREFIX).unwrap();

    info!("{} Start rectifier... 🫨", now!());

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

    let config_path = get_cur_dir_file(INIT_CONFIG).unwrap();
    let config_path = config_path.to_string_lossy();
    let config: InitiationsConfig = read_yaml(config_path).unwrap();

    match args.option.as_str() {
        DEL_QUE => {
            info!("{} DEL_QUE", now!());
            delete_queues(&mq_client, &config).await.unwrap();
        }
        DEL_EXG => {
            info!("{} DEL_EXG", now!());
            delete_exchanges(&mq_client, &config).await.unwrap();
        }
        DEL_TBL => {
            info!("{} DEL_TBL", now!());
            delete_tables(&db_client).await.unwrap();
        }
        _ => panic!("undefined option"),
    }

    info!("{} End rectifier 😎", now!());
}
