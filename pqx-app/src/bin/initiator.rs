//! file: initiator.rs
//! author: Jacob Xie
//! date: 2023/06/17 15:54:50 Saturday
//! brief:

use clap::Parser;
use pqx::amqprs::channel::{ExchangeType, QueueBindArguments};
use pqx::error::PqxResult;
use pqx::mq::{FieldTableBuilder, MqClient};
use pqx::pqx_util::*;
use pqx_app::cfg::{ConnectionsConfig, InitiationsConfig};
use pqx_app::persistence::MessagePersistent;
use tracing::info;

// ================================================================================================
// Const
// ================================================================================================

// commands
const INSP: &str = "insp";
const DECL_X: &str = "decl_x";
const DECL_DX: &str = "decl_dx";
const DECL_DLX: &str = "decl_dlx";
const CRT_TBL: &str = "crt_tbl";
const DECL_ALL: &str = "decl_all";

// default constants
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
) -> PqxResult<(serde_json::Value, serde_json::Value, serde_json::Value)> {
    let query = MqQuery::new(&client);

    let res1 = query.exchanges_with_vhost(client.vhost()).await?;
    let res2 = query.queues_with_vhost(&client.vhost()).await?;
    let res3 = query.bindings_with_vhost(&client.vhost()).await?;

    Ok((res1, res2, res3))
}

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
/// 0. cargo run --bin initiator -- -o insp
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

    let _guard = logging_init(LOGGING_DIR, FILENAME_PREFIX).unwrap();

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

    // read setup config
    let config_path = get_cur_dir_file(INIT_CONFIG).unwrap();
    let config_path = config_path.to_string_lossy();
    let config: InitiationsConfig = read_yaml(config_path).unwrap();

    match args.option.as_str() {
        INSP => {
            // check tables
            let res = check_tables(&db_client).await.unwrap();

            info!("{} {:?}", now!(), res);

            // check exchanges, queues and bindings exist
            let (res1, res2, res3) = check_mq(&api_client).await.unwrap();
            info!("{} {:?}", now!(), &res1);
            info!("{} {:?}", now!(), &res2);
            info!("{} {:?}", now!(), &res3);
        }
        DECL_X => declare_exchange_and_queues_then_bind(&mq_client, &config)
            .await
            .unwrap(),
        DECL_DX => declare_delayed_exchange_and_bind_queues(&mq_client, &config)
            .await
            .unwrap(),
        DECL_DLX => declare_dead_letter_exchange_and_bind_queues(&mq_client, &config)
            .await
            .unwrap(),
        CRT_TBL => create_table(&db_client).await.unwrap(),
        DECL_ALL => {
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
