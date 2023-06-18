//! file: initiator.rs
//! author: Jacob Xie
//! date: 2023/06/17 15:54:50 Saturday
//! brief:

use clap::Parser;
use pqx::amqprs::channel::{ExchangeType, QueueBindArguments};
use pqx::ec::util::*;
use pqx::error::PqxResult;
use pqx::mq::{FieldTableBuilder, MqClient};
use pqx::pqx_util::{logging_init, read_yaml};
use pqx_app::cfg::{ConnectionsConfig, InitiationsConfig};

// ================================================================================================
// Const
// ================================================================================================

const LOGGING_DIR: &str = ".";
const FILENAME_PREFIX: &str = "pqx_initiator";
const CONN_CONFIG: &str = "conn.yml";
const INIT_CONFIG: &str = "init.yml";

// ================================================================================================
// Cfg & Args
// ================================================================================================

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    option: String,
}

// ================================================================================================
// Helper
// ================================================================================================

// PANIC if file not found!
fn get_conn_yaml_path(filename: &str) -> std::path::PathBuf {
    join_dir(current_dir().unwrap(), filename).unwrap()
}

// ================================================================================================
// Fn
// ================================================================================================

async fn declare_exchange_and_queues_then_bind(
    client: &MqClient,
    config: &InitiationsConfig,
) -> PqxResult<()> {
    // declare queues
    client
        .declare_exchange(&config.exchange, &ExchangeType::Headers)
        .await?;

    // declare queues and bind to exchange
    for hq in &config.header_queues {
        // declare queue
        client.declare_queue(&hq.queue).await?;
        // bind queue to exchange
        let mut args = QueueBindArguments::new(&hq.queue, &config.exchange, "");
        let mut headers = FieldTableBuilder::new();
        headers.x_match(&hq.match_type);
        for kv in hq.kv.iter() {
            headers.x_common_pair(kv.key.clone(), kv.value.clone());
        }
        args.arguments(headers.finish());
        client.bind_queue_by_args(args).await?;
    }

    Ok(())
}

async fn declare_delayed_exchange_and_bind_queues(
    client: &MqClient,
    config: &InitiationsConfig,
) -> PqxResult<()> {
    // declare delayed exchange
    client
        .declare_exchange(&config.delayed_exchange, &ExchangeType::Headers)
        .await?;

    // bind existing queues to delayed exchange (suppose queue has already been declared in the former step)
    for hq in &config.header_queues {
        client
            .bind_queue(&config.delayed_exchange, "", &hq.queue)
            .await?;
    }

    Ok(())
}

async fn declare_dead_letter_exchange_and_bind_queues(
    client: &MqClient,
    config: &InitiationsConfig,
) -> PqxResult<()> {
    // declare dead letter exchange
    client
        .declare_exchange(&config.dead_letter_exchange, &ExchangeType::Direct)
        .await?;

    // bind existing queues to dead letter exchange (suppose queue has already been declared in the former step)
    for hq in &config.header_queues {
        client
            .bind_queue(&config.dead_letter_exchange, "", &hq.queue)
            .await?;
    }

    Ok(())
}

// ================================================================================================
// Main
// ================================================================================================

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    logging_init(LOGGING_DIR, FILENAME_PREFIX);

    // read connection config
    let config_path = get_conn_yaml_path(CONN_CONFIG);
    let config_path = config_path.to_string_lossy();
    let config: ConnectionsConfig = read_yaml(config_path).unwrap();
    let mut client = MqClient::new();
    client.connect(config.mq).await.unwrap();

    // read setup config
    let config_path = get_conn_yaml_path(INIT_CONFIG);
    let config_path = config_path.to_string_lossy();
    let config: InitiationsConfig = read_yaml(config_path).unwrap();

    match args.option.as_str() {
        "decl_x" => declare_exchange_and_queues_then_bind(&client, &config)
            .await
            .unwrap(),
        "decl_dx" => declare_delayed_exchange_and_bind_queues(&client, &config)
            .await
            .unwrap(),
        "decl_dlx" => declare_dead_letter_exchange_and_bind_queues(&client, &config)
            .await
            .unwrap(),
        "all" => {
            declare_exchange_and_queues_then_bind(&client, &config)
                .await
                .unwrap();
            declare_delayed_exchange_and_bind_queues(&client, &config)
                .await
                .unwrap();
            declare_dead_letter_exchange_and_bind_queues(&client, &config)
                .await
                .unwrap();
        }
        _ => panic!("undefined option"),
    }
}
