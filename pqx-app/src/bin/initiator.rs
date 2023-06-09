//! file: initiator.rs
//! author: Jacob Xie
//! date: 2023/06/17 15:54:50 Saturday
//! brief:

use clap::Parser;
use pqx::amqprs::channel::{ExchangeType, QueueBindArguments, QueueDeclareArguments};
use pqx::error::PqxResult;
use pqx::mq::{FieldTableBuilder, MqClient, EXCHANGE_TYPE_DELAYED};
use pqx::pqx_util::*;
use pqx_app::cfg::{ConnectionsConfig, InitiationsConfig};
use pqx_app::persist::MessagePersistent;
use tracing::info;

// ================================================================================================
// Const
// ================================================================================================

// commands
const DECL_X: &str = "decl_x";
const DECL_DX: &str = "decl_dx";
const DECL_DLX: &str = "decl_dlx";
const CRT_TBL: &str = "crt_tbl";
const INIT: &str = "init";

// default constants
const LOGGING_DIR: &str = "./logs";
const FILENAME_PREFIX: &str = "pqx_initiator";
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

async fn declare_exchange_and_queues_then_bind(
    client: &MqClient,
    config: &InitiationsConfig,
) -> PqxResult<()> {
    // declare queues
    client
        .declare_exchange(&config.header_exchange, &ExchangeType::Headers)
        .await?;

    // declare queues and bind to exchange
    for hq in &config.header_queues {
        // declare queue
        let mut args = QueueDeclareArguments::new(&hq.queue);
        let mut headers = FieldTableBuilder::new();
        // set dead letter exchange
        headers.x_dead_letter_exchange(&config.dead_letter_exchange, "");
        args.arguments(headers.finish());
        client.declare_queue_by_args(args).await?;

        // bind queue to exchange
        let mut args = QueueBindArguments::new(&hq.queue, &config.header_exchange, "");
        let mut headers = FieldTableBuilder::new();
        // set "x-match"
        headers.x_match(&hq.match_type);
        // set matching pattern
        for (k, v) in hq.kv.iter() {
            headers.x_common_pair(k, v);
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
    let mut args = FieldTableBuilder::new();
    args.x_delayed_type(&ExchangeType::Headers);
    client
        .declare_exchange_with_args(
            &config.delayed_exchange,
            &EXCHANGE_TYPE_DELAYED,
            args.finish(),
        )
        .await?;

    // bind existing queues to delayed exchange (suppose queue has already been declared in the former step)
    for hq in &config.header_queues {
        let mut args = QueueBindArguments::new(&hq.queue, &config.delayed_exchange, "");
        let mut headers = FieldTableBuilder::new();
        headers.x_match(&hq.match_type);
        for (k, v) in hq.kv.iter() {
            headers.x_common_pair(k, v);
        }
        args.arguments(headers.finish());

        client.bind_queue_by_args(args).await?;
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

    // declare dead letter queue and bind to dead letter exchange
    let mut args = QueueDeclareArguments::new(&config.dead_letter_queue);
    let mut headers = FieldTableBuilder::new();
    // set "x-message-ttl"
    if let Some(ttl) = &config.dead_message_ttl {
        headers.x_consume_ttl(*ttl);
    }
    args.arguments(headers.finish());
    client.declare_queue_by_args(args).await?;
    client
        .bind_queue(&config.dead_letter_exchange, "", &config.dead_letter_queue)
        .await?;

    Ok(())
}

async fn create_table(client: &PersistClient) {
    let db = client.db().expect("connection is established");
    let mp = MessagePersistent::new(db.clone());

    mp.create_table().await;
}

// ================================================================================================
// Main
// ================================================================================================

/// Options
///
/// 1. cargo run --bin initiator -- -o decl_x
/// 2. cargo run --bin initiator -- -o decl_dx
/// 3. cargo run --bin initiator -- -o decl_dlx
/// 4. cargo run --bin initiator -- -o crt_tbl
/// 4. cargo run --bin initiator -- -o init
///
#[tokio::main]
async fn main() {
    let args = Args::parse();

    let _guard = logging_init(LOGGING_DIR, FILENAME_PREFIX, tracing::Level::INFO).unwrap();

    info!("{} Start initiator... 🫨", now!());

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

    // read setup config
    let config_path = get_cur_dir_file(INIT_CONFIG).unwrap();
    let config_path = config_path.to_string_lossy();
    let config: InitiationsConfig = read_yaml(config_path).unwrap();

    match args.option.as_str() {
        DECL_X => {
            info!("{} DECL_X", now!());
            declare_exchange_and_queues_then_bind(&mq_client, &config)
                .await
                .unwrap();
        }
        DECL_DX => {
            info!("{} DECL_DX", now!());
            declare_delayed_exchange_and_bind_queues(&mq_client, &config)
                .await
                .unwrap();
        }
        DECL_DLX => {
            info!("{} DECL_DLX", now!());
            declare_dead_letter_exchange_and_bind_queues(&mq_client, &config)
                .await
                .unwrap();
        }
        CRT_TBL => {
            info!("{} CRT_TBL", now!());
            create_table(&db_client).await;
        }
        INIT => {
            info!("{} INIT", now!());
            declare_exchange_and_queues_then_bind(&mq_client, &config)
                .await
                .unwrap();
            declare_delayed_exchange_and_bind_queues(&mq_client, &config)
                .await
                .unwrap();
            declare_dead_letter_exchange_and_bind_queues(&mq_client, &config)
                .await
                .unwrap();
            create_table(&db_client).await;
        }
        _ => panic!("undefined option"),
    }

    info!("{} End initiator 😎", now!());
}
