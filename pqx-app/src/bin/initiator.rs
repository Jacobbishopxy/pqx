//! file: initiator.rs
//! author: Jacob Xie
//! date: 2023/06/17 15:54:50 Saturday
//! brief:

use clap::Parser;
use pqx::amqprs::channel::{ExchangeType, QueueBindArguments};
use pqx::error::PqxResult;
use pqx::mq::{FieldTableBuilder, MqClient};
use pqx::pqx_util::get_cur_dir_file;
use pqx::pqx_util::{logging_init, read_yaml, PersistClient};
use pqx_app::cfg::{ConnectionsConfig, InitiationsConfig};
use pqx_app::persistence::MessagePersistent;

// ================================================================================================
// Const
// ================================================================================================

const LOGGING_DIR: &str = "./logs";
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
        client.declare_queue(&hq.queue).await?;
        // bind queue to exchange
        let mut args = QueueBindArguments::new(&hq.queue, &config.header_exchange, "");
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

    // bind existing queues to dead letter exchange (suppose queue has already been declared in the former step)
    for hq in &config.header_queues {
        client
            .bind_queue(&config.dead_letter_exchange, "", &hq.queue)
            .await?;
    }

    Ok(())
}

async fn create_table(client: &PersistClient) -> PqxResult<()> {
    let db = client.db().expect("connection is established");
    let mp = MessagePersistent::new(db.clone());

    mp.create_table().await?;

    Ok(())
}

// ================================================================================================
// Main
// ================================================================================================

/// Main
///
/// 1. cargo run --bin initiator -- -o decl_x
/// 2. cargo run --bin initiator -- -o decl_dx
/// 3. cargo run --bin initiator -- -o decl_dlx
/// 4. cargo run --bin initiator -- -o decl_all
/// 5. cargo run --bin initiator -- -o crt_tbl
/// 6. cargo run --bin initiator -- -o all
///
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    logging_init(LOGGING_DIR, FILENAME_PREFIX);

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
    db_client.connect().await.unwrap();

    // read setup config
    let config_path = get_cur_dir_file(INIT_CONFIG).unwrap();
    let config_path = config_path.to_string_lossy();
    let config: InitiationsConfig = read_yaml(config_path).unwrap();

    match args.option.as_str() {
        "decl_x" => declare_exchange_and_queues_then_bind(&mq_client, &config)
            .await
            .unwrap(),
        "decl_dx" => declare_delayed_exchange_and_bind_queues(&mq_client, &config)
            .await
            .unwrap(),
        "decl_dlx" => declare_dead_letter_exchange_and_bind_queues(&mq_client, &config)
            .await
            .unwrap(),
        "crt_tbl" => create_table(&db_client).await.unwrap(),
        "decl_all" => {
            declare_exchange_and_queues_then_bind(&mq_client, &config)
                .await
                .unwrap();
            declare_delayed_exchange_and_bind_queues(&mq_client, &config)
                .await
                .unwrap();
            declare_dead_letter_exchange_and_bind_queues(&mq_client, &config)
                .await
                .unwrap();
            create_table(&db_client).await.unwrap();
        }
        _ => panic!("undefined option"),
    }
}
