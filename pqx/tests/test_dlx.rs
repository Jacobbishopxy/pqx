//! file: test_dlx.rs
//! author: Jacob Xie
//! date: 2023/05/30 22:20:25 Tuesday
//! brief: test dead letter exchange
//! ref: https://www.cloudamqp.com/blog/when-and-how-to-use-the-rabbitmq-dead-letter-exchange.html
//! process:
//! 1. declare dlx, and queue
//! 2. declare normal exchange and queue with dlx specified
//! 3. publish message

use amqprs::channel::*;
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use pqx::mq::*;
use pqx_util::get_cur_dir_file;
use serde::{Deserialize, Serialize};

// ================================================================================================
// const
// ================================================================================================

const EXCHG: &str = "pqx.test.direct";
const ROUT: &str = "pqx.test.rout";
const QUE: &str = "pqx.test.que";

const DLX: &str = "pqx.test.dlx";
const DL_ROUT: &str = "pqx.test.dl-rout";
const DL_QUE: &str = "pqx.test.dl-que";

// ================================================================================================
// msg
// ================================================================================================

#[derive(Debug, Serialize, Deserialize)]
struct DevMsg {
    to_dlx: bool,
    data: String,
    time: DateTime<Local>,
}

impl DevMsg {
    fn new(to_dlx: bool, data: &str) -> Self {
        Self {
            to_dlx,
            data: data.to_string(),
            time: Local::now(),
        }
    }
}

// ================================================================================================
// impl AsyncConsumer
// ================================================================================================

#[derive(Clone)]
struct DevDlxConsumer;

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
            println!("msg cannot be recognized, direct into dlx : {:?}", res);
            return;
        };

        // explicit send msg to dlx
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
// test
// ================================================================================================

#[tokio::test]
async fn declare_dlx_success() {
    /*
    cargo test --package pqx --test test_dlx -- declare_dlx_success --exact --nocapture
     */

    // 0. client connection and open channel
    let mut client = MqClient::new();
    let pth = get_cur_dir_file("conn.yml").unwrap();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 1. declare an exchange for dead letter
    let res = client.declare_exchange(DLX, &ExchangeType::Direct).await;
    assert!(res.is_ok());

    // 2. declare a queue and bind to the dead letter exchange
    let res = client.declare_and_bind_queue(DLX, DL_ROUT, DL_QUE).await;
    assert!(res.is_ok());
    println!("queue info: {:?}", res.unwrap());
}

#[tokio::test]
async fn mq_subscribe_success() {
    /*
    cargo test --package pqx --test test_dlx -- mq_subscribe_success --exact --nocapture
     */

    // 0. client connection and open channel
    let mut client = MqClient::new();
    let pth = get_cur_dir_file("conn.yml").unwrap();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 1. declare a normal exchange
    let res = client.declare_exchange(EXCHG, &ExchangeType::Direct).await;
    assert!(res.is_ok());
    let chan = client.channel().unwrap();

    // 2. set qos (quality of service)
    let args = BasicQosArguments::new(0, 1, true);
    let res = chan.basic_qos(args).await;
    assert!(res.is_ok());

    // 3. declare a queue with args which explicit specified "x-dead-letter-exchange" to dlx,
    // "x-dead-letter-routing-key" to dlx routing key, and bind to the declared exchange
    // [temporary workaround] `insert`
    let res = client
        .declare_queue_with_dlx(QUE, DLX, DL_ROUT, Some(6000))
        .await;
    println!("{:?}", res);
    assert!(res.is_ok());

    let res = client.bind_queue(EXCHG, ROUT, QUE).await;
    assert!(res.is_ok());

    // 4. consume
    let consumer = DevDlxConsumer;
    let mut subscriber = BasicSubscriber::new(chan, consumer);

    let res = subscriber.consume(QUE).await;
    assert!(res.is_ok());
    println!("Start listening on {}:{} ...", "HOST", "PORT");
    subscriber.block().await;
}

#[tokio::test]
async fn publish_msg_to_dlx_success() {
    /*
    cargo test --package pqx --test test_dlx -- publish_msg_to_dlx_success --exact --nocapture
     */

    // 0. client connection and open channel
    let mut client = MqClient::new();
    let pth = get_cur_dir_file("conn.yml").unwrap();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());
    let chan = client.channel().unwrap();

    // 1. publisher
    let publisher = Publisher::new(chan);

    // 2. send msg
    let msg = DevMsg::new(true, "I'm going into dlx."); // explicit to dlx
    let res = publisher.publish(EXCHG, ROUT, msg).await;
    assert!(res.is_ok());
    let msg = DevMsg::new(false, "Normal data.");
    let res = publisher.publish(EXCHG, ROUT, msg).await;
    assert!(res.is_ok());

    // 3. block until msg has been sent
    publisher.block(1).await;
}
