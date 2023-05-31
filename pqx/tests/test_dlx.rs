//! file: test_dlx.rs
//! author: Jacob Xie
//! date: 2023/05/30 22:20:25 Tuesday
//! brief:

use std::sync::Arc;

use amqprs::channel::{
    BasicAckArguments, BasicNackArguments, Channel, ExchangeType, QueueDeclareArguments,
};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver, FieldTable};
use async_trait::async_trait;
use pqx::ec::util::*;
use pqx::ec::*;
use pqx::error::PqxResult;
use pqx::mq::*;

// ================================================================================================
// const
// ================================================================================================

const EXCHG: &str = "amq.direct";
const ROUT: &str = "rbmq-rs-rout";
const QUE: &str = "rbmq-rs-que";
const TAG: &str = "rbmq-rs-tag";

const DLX: &str = "rbmq-rs-dlx";
const DL: &str = "rbmq-rs-dl";

// ================================================================================================
// help
// ================================================================================================

fn get_conn_yaml_path() -> std::path::PathBuf {
    join_dir(current_dir().unwrap(), "conn.yml").unwrap()
}

async fn a_print_stdout(s: String) -> PqxResult<()> {
    println!("print_stdout: {:?}", s);

    Ok(())
}

async fn a_print_stderr(s: String) -> PqxResult<()> {
    println!("print_stderr: {:?}", s);

    Ok(())
}

// ================================================================================================
// impl AsyncConsumer
// ================================================================================================

#[derive(Clone)]
struct DevDlxConsumer {
    executor: CmdAsyncExecutor,
}

impl DevDlxConsumer {
    fn new() -> Self {
        Self {
            executor: CmdAsyncExecutor::new(),
        }
    }
}

#[async_trait]
impl AsyncConsumer for DevDlxConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {

        // TODO: should recognize from content to see which message to be sent to dlx
    }
}

// ================================================================================================
// subscriber
// ================================================================================================

#[tokio::test]
async fn declare_dlx_success() {
    let mut client = MqClient::new();

    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());

    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 1. declare an exchange for dead letter
    let res = client.declare_exchange(DLX, ExchangeType::Direct).await;
    assert!(res.is_ok());

    // 2. declare a queue with args which explicit specified a key "x-dead-letter-routing-key"
    // [temporary workaround]
    let mut args = QueueDeclareArguments::new(QUE);
    let mut ft = FieldTable::new();
    ft.insert(
        "x-dead-letter-routing-key".try_into().unwrap(),
        DLX.try_into().unwrap(),
    );
    args.arguments(ft);
    let res = client.declare_queue_with_args(DLX, args).await;
}

#[tokio::test]
async fn publish_msg_to_dlx_success() {
    let mut client = MqClient::new();

    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());

    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    let publisher = Publisher::new(client.channel().unwrap());

    // TODO: msg with dlx mark, should be recognized in [`DevDlxConsumer`]
    // let res = publisher.publish(DLX, rout, msg)
}

#[tokio::test]
async fn mq_subscribe_success() {
    let mut client = MqClient::new();

    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());

    let res = client.declare_and_bind_queue(EXCHG, ROUT, QUE).await;
    assert!(res.is_ok());

    // TODO:
    // let res = client.declare_and_bind_queue(DLX, rout, que)
}
