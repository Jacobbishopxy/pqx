//! file: test_dlx.rs
//! author: Jacob Xie
//! date: 2023/05/30 22:20:25 Tuesday
//! brief: test dead letter exchange
//! ref: https://www.cloudamqp.com/blog/when-and-how-to-use-the-rabbitmq-dead-letter-exchange.html
//! process:
//! 1. declare dlx, and queue
//! 2. declare normal exchange and queue with dlx specified
//! 3. publish message

use std::sync::Arc;

use amqprs::channel::*;
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver, FieldTable};
use async_trait::async_trait;
use pqx::ec::util::*;
use pqx::ec::*;
use pqx::error::PqxResult;
use pqx::mq::*;
use serde::{Deserialize, Serialize};

// ================================================================================================
// const
// ================================================================================================

const EXCHG: &str = "rbmq-rs-exchange";
const ROUT: &str = "rbmq-rs-rout";
const QUE: &str = "rbmq-rs-que";
const TAG: &str = "rbmq-rs-tag";

const DLX: &str = "rbmq-rs-dlx";
const DL_ROUT: &str = "rbmq-rs-dl-rout";
const DL_QUE: &str = "rbmq-rs-dl-queue";

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
// msg
// ================================================================================================

#[derive(Debug, Serialize, Deserialize)]
struct DevMsg {
    to_dlx: bool,
    data: String,
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
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let headers = basic_properties.headers();
        println!("headers: {:?}", headers);

        let msg = if let Ok(m) = serde_json::from_slice::<DevMsg>(&content) {
            println!("message: {:?}", m);
            m
        } else {
            // if not a `DevMsg`, then throw it into dlx (`requeue=false`)
            let args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
            let res = channel.basic_nack(args).await;
            println!("msg cannot be recognozed, direct into dlx : {:?}", res);
            return;
        };

        if msg.to_dlx {
            let args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
            let res = channel.basic_nack(args).await;
            println!("nack: {:?}", res);
        } else {
            let args = BasicAckArguments::new(deliver.delivery_tag(), false);
            let res = channel.basic_ack(args).await;
            println!("ack: {:?}", res);
        }
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

    // 2. declare a queue and bind to the dead letter exchange
    let res = client.declare_and_bind_queue(DLX, DL_ROUT, DL_QUE).await;
    assert!(res.is_ok());
    println!("queue info: {:?}", res.unwrap());
}

#[tokio::test]
async fn mq_subscribe_success() {
    let mut client = MqClient::new();

    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 1. declare a normal exchange
    let res = client.declare_exchange(EXCHG, ExchangeType::Direct).await;
    assert!(res.is_ok());
    let chan = client.channel().unwrap();

    // 2. set qos
    let args = BasicQosArguments::new(0, 1, true);
    let res = chan.basic_qos(args).await;
    assert!(res.is_ok());

    // 3. declare a queue with args which explicit specified "x-dead-letter-exchange" to dlx,
    // "x-dead-letter-routing-key" to dlx routing key, and bind to the declared exchange
    // [temporary workaround] `insert`
    let mut args = QueueDeclareArguments::new(QUE);
    let mut ft = FieldTable::new();
    ft.insert(
        "x-dead-letter-exchange".try_into().unwrap(),
        DLX.try_into().unwrap(),
    );
    ft.insert(
        "x-dead-letter-routing-key".try_into().unwrap(),
        DL_ROUT.try_into().unwrap(),
    );
    // ft.insert(
    //     "x-message-ttl".try_into().unwrap(),
    //     "60000".try_into().unwrap(),
    // );
    args.arguments(ft);
    let res = client.declare_queue_with_args(args).await;
    println!("{:?}", res);
    assert!(res.is_ok());

    let res = client.bind_queue(EXCHG, ROUT, QUE).await;
    assert!(res.is_ok());

    // 4. consume
    let mut consumer = DevDlxConsumer::new();
    consumer
        .executor
        .register_stdout_fn(Arc::new(a_print_stdout));
    consumer
        .executor
        .register_stderr_fn(Arc::new(a_print_stderr));
    let subscriber = Subscriber::new(chan, consumer);

    let res = subscriber.consume(QUE, TAG).await;
    assert!(res.is_ok());
    println!("Start listening on {}:{} ...", "HOST", "PORT");
    subscriber.block().await;
}

#[tokio::test]
async fn publish_msg_to_dlx_success() {
    let mut client = MqClient::new();

    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());
    let chan = client.channel().unwrap();

    let publisher = Publisher::new(chan);

    let msg = DevMsg {
        to_dlx: true,
        data: String::from("I'm going into dlx."),
    };

    let res = publisher.publish(EXCHG, ROUT, msg).await;
    assert!(res.is_ok());

    publisher.block(1).await;
}
