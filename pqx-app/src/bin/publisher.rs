//! file: publisher.rs
//! author: Jacob Xie
//! date: 2023/06/25 09:36:46 Sunday
//! brief: turn `task.json` into `Command` and send to MQ

use clap::Parser;
use pqx::amqprs::BasicProperties;
use pqx::mq::{MqClient, Publisher};
use pqx::pqx_util::*;
use pqx_app::adt::Command;
use pqx_app::cfg::{ConnectionsConfig, InitiationsConfig};
use tracing::info;

// ================================================================================================
// Const
// ================================================================================================

// commands
const PUB: &str = "pub";

// default constants
const LOGGING_DIR: &str = "./logs";
const FILENAME_PREFIX: &str = "pqx_publisher";
const CONN_CONFIG: &str = "conn.yml";
const INIT_CONFIG: &str = "init.yml";
const TASK: &str = "task.json";

// ================================================================================================
// Args
// ================================================================================================

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    option: String,
    config: Option<String>,
    task: Option<String>,
}

// ================================================================================================
// Fn
// ================================================================================================

// ================================================================================================
// Main
// ================================================================================================

/// 0. cargo run --bin publisher -- -o pub
#[tokio::main]
async fn main() {
    let args = Args::parse();

    let _guard = logging_init(LOGGING_DIR, FILENAME_PREFIX, tracing::Level::INFO).unwrap();

    info!("{} Start publisher... 🫨", now!());

    // read connection config
    let config_path = get_cur_dir_file(CONN_CONFIG).unwrap();
    let config_path = config_path.to_string_lossy();
    let conn_config: ConnectionsConfig = read_yaml(config_path).unwrap();

    // read setup config
    let config_path = get_cur_dir_file(INIT_CONFIG).unwrap();
    let config_path = config_path.to_string_lossy();
    let init_config: InitiationsConfig = read_yaml(config_path).unwrap();

    // mq client
    let mut mq_client = MqClient::new();
    mq_client.connect(conn_config.mq).await.unwrap();
    mq_client.open_channel(None).await.unwrap();
    let chan = mq_client.channel().unwrap();

    // read task.json
    let task_path = get_cur_dir_file(args.task.as_deref().unwrap_or(TASK)).unwrap();
    let task_path = task_path.to_string_lossy();
    let task: Command = read_json(task_path).unwrap();

    // publisher
    let publisher = Publisher::new(chan);

    match args.option.as_str() {
        PUB => {
            let props_list = Vec::<BasicProperties>::try_from(&task).unwrap();
            for props in props_list.into_iter() {
                publisher
                    .publish_with_props(&init_config.header_exchange, "", task.cmd().clone(), props)
                    .await
                    .unwrap();
            }
        }
        _ => panic!("undefined option"),
    }

    publisher.block(1).await;

    info!("{} End publisher 😎", now!());
}
