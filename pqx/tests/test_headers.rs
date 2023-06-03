//! file: test_headers.rs
//! author: Jacob Xie
//! date: 2023/06/03 15:23:11 Saturday
//! brief:

use amqprs::channel::*;
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use amqprs::{FieldName, FieldTable, FieldValue};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use pqx::ec::util::*;
use pqx::mq::*;
use serde::{Deserialize, Serialize};

// ================================================================================================
// const
// ================================================================================================

const EXCHG: &str = "rbmq-rs-header";

const HEADER_QUE1: &str = "rbmq-rs-hq1";
const HEADER_QUE2: &str = "rbmq-rs-hq2";

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
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        println!("basic_properties: {:?}", basic_properties.headers());

        match serde_json::from_slice::<serde_json::Value>(&content) {
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
    let res = client.declare_exchange(EXCHG, ExchangeType::Headers).await;
    assert!(res.is_ok());

    // 2. declare and bind queues
    // declare que1
    let args = QueueDeclareArguments::new(HEADER_QUE1);
    let res = client.declare_queue(args).await;
    assert!(res.is_ok());
    // bind que1, with "x-match" and other pattern args
    let mut args = QueueBindArguments::new(HEADER_QUE1, EXCHG, "");
    let mut ft = FieldTable::new();
    ft.insert(
        FieldName::try_from("x-match").unwrap(),
        FieldValue::from(String::from("any")),
    );
    ft.insert(
        FieldName::try_from("unique_key").unwrap(),
        FieldValue::from(String::from("u1")),
    );
    ft.insert(
        FieldName::try_from("common_key").unwrap(),
        FieldValue::from(String::from("c1")),
    );
    args.arguments(ft);
    let res = client.bind_queue(args).await;
    assert!(res.is_ok());

    // declare que2
    let args = QueueDeclareArguments::new(HEADER_QUE2);
    let res = client.declare_queue(args).await;
    assert!(res.is_ok());
    // bind que2, with "x-match" and other pattern args
    let mut args = QueueBindArguments::new(HEADER_QUE2, EXCHG, "");
    let mut ft = FieldTable::new();
    ft.insert(
        FieldName::try_from("x-match").unwrap(),
        FieldValue::from(String::from("any")),
    );
    ft.insert(
        FieldName::try_from("unique_key").unwrap(),
        FieldValue::from(String::from("u2")),
    );
    ft.insert(
        FieldName::try_from("common_key").unwrap(),
        FieldValue::from(String::from("c1")),
    );
    args.arguments(ft);
    let res = client.bind_queue(args).await;
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

    let res = subscriber.consume(HEADER_QUE1).await;
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

    let res = subscriber.consume(HEADER_QUE2).await;
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
    // `unique_key=1`
    let mut props = BasicProperties::default();
    let mut tf = FieldTable::new();
    tf.insert(
        FieldName::try_from("unique_key").unwrap(),
        FieldValue::from(String::from("u1")),
    );
    props.with_headers(tf);
    let msg = DevMsg::new("Sending to unique_key=1");
    let res = publisher.publish_with_props(EXCHG, "", msg, props).await;
    assert!(res.is_ok());

    // `unique_key=2`
    let mut props = BasicProperties::default();
    let mut tf = FieldTable::new();
    tf.insert(
        FieldName::try_from("unique_key").unwrap(),
        FieldValue::from(String::from("h2")),
    );
    props.with_headers(tf);
    let msg = DevMsg::new("Sending to unique_key=2");
    let res = publisher.publish_with_props(EXCHG, "", msg, props).await;
    assert!(res.is_ok());

    // `common_key=a`
    let mut props = BasicProperties::default();
    let mut tf = FieldTable::new();
    tf.insert(
        FieldName::try_from("common_key").unwrap(),
        FieldValue::from(String::from("c1")),
    );
    props.with_headers(tf);
    let msg = DevMsg::new("Sending to common_key=a");
    let res = publisher.publish_with_props(EXCHG, "", msg, props).await;
    assert!(res.is_ok());

    // 3. block until msg has been sent
    publisher.block(1).await;
}
