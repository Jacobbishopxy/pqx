//! file: test_topics.rs
//! author: Jacob Xie
//! date: 2023/06/02 08:47:46 Friday
//! brief:

use amqprs::channel::*;
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use pqx::ec::util::*;
use pqx::mq::*;
use serde::{Deserialize, Serialize};

// ================================================================================================
// const
// ================================================================================================

const EXCHG: &str = "rbmq-rs-topic";

const TOPIC_ROUT1: &str = "rbmq-rs-rout.tag1.*";
const TOPIC_ROUT2: &str = "rbmq-rs-rout.*.tag2";

const TOPIC_QUE1: &str = "rbmq-rs-tq1";
const TOPIC_QUE2: &str = "rbmq-rs-tq2";

// ================================================================================================
// helper
// ================================================================================================

fn get_conn_yaml_path() -> std::path::PathBuf {
    join_dir(current_dir().unwrap(), "conn.yml").unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
struct DevMsg {
    data: String,
    time: DateTime<Local>,
}

impl DevMsg {
    fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
            time: Local::now(),
        }
    }
}

// ================================================================================================
// impl AsyncConsumer
// ================================================================================================

#[derive(Clone)]
struct DevTopicConsumer;

#[async_trait]
impl AsyncConsumer for DevTopicConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        match serde_json::from_slice::<DevMsg>(&content) {
            Ok(m) => {
                let args = BasicAckArguments::new(deliver.delivery_tag(), false);
                let _ = channel.basic_ack(args).await;
                println!("ack: {:?}", m);
            }
            Err(_) => {
                // if not a `DevMsg`, then discard (`requeue=false`)
                let args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
                let res = channel.basic_nack(args).await;
                println!("msg cannot be recognized, discarded: {:?}", res);
            }
        };

        // mock process time, wait 10 secs
        tokio::time::sleep(tokio::time::Duration::new(10, 0)).await;
    }
}

// ================================================================================================
// test
// ================================================================================================

#[tokio::test]
async fn declare_exchange_and_queues() {
    // 0. client connection and open channel
    let mut client = MqClient::new();
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 1. declare exchange
    let res = client.declare_exchange(EXCHG, ExchangeType::Topic).await;
    assert!(res.is_ok());

    // 2. declare and bind queues
    let res = client
        .declare_and_bind_queue(EXCHG, TOPIC_ROUT1, TOPIC_QUE1)
        .await;
    assert!(res.is_ok());
    let res = client
        .declare_and_bind_queue(EXCHG, TOPIC_ROUT2, TOPIC_QUE2)
        .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn mq_subscribe1_success() {
    // 0. client connection and open channel
    let mut client = MqClient::new();
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());
    let chan = client.channel().unwrap();

    // 1. consume
    let consumer = DevTopicConsumer;
    let mut subscriber = Subscriber::new(chan, consumer);

    let res = subscriber.consume(TOPIC_QUE1).await;
    assert!(res.is_ok());

    subscriber.block().await;
}

#[tokio::test]
async fn mq_subscribe2_success() {
    // 0. client connection and open channel
    let mut client = MqClient::new();
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());
    let chan = client.channel().unwrap();

    // 1. consume
    let consumer = DevTopicConsumer;
    let mut subscriber = Subscriber::new(chan, consumer);

    let res = subscriber.consume(TOPIC_QUE2).await;
    assert!(res.is_ok());

    subscriber.block().await;
}

#[tokio::test]
async fn publish_msg_route() {
    // 0. client connection and open channel
    let mut client = MqClient::new();
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());
    let chan = client.channel().unwrap();

    // 1. publisher
    let publisher = Publisher::new(chan);

    // 2. send msg
    let msg = DevMsg::new("Sending to tag1");
    let res = publisher.publish(EXCHG, "rbmq-rs-rout.tag1.xxx", msg).await;
    assert!(res.is_ok());
    let msg = DevMsg::new("Sending to tag2");
    let res = publisher.publish(EXCHG, "rbmq-rs-rout.xxx.tag2", msg).await;
    assert!(res.is_ok());

    // 3. block until msg has been sent
    publisher.block(1).await;
}
