//! file: test_delay_retry.rs
//! author: Jacob Xie
//! date: 2023/06/10 08:21:56 Saturday
//!
//! Article: https://engineering.nanit.com/rabbitmq-retries-the-full-story-ca4cc6c5b493
//! Plugin: https://github.com/rabbitmq/rabbitmq-delayed-message-exchange
//!
//! This test case is based on the 'Option 4' of the article mentioned above:
//!
//! 1. mailman consumer receives a message from RabbitMQ and fails processing;
//! 2. It then ACK's the original message and publishes it to the delayed exchange with an
//! incremented `x-retires` header, a calculated `x-delay` header to have the message delayed
//! before it is being forwarded on and a routing key matching the name of the queue the
//! message originated from `mailman.users.created`;
//! 3. When the TTL expires the delayed exchange forwards the message back to the queue
//! `mailman.users.created` which is attached to it via a routing key of its name;
//! 4. mailman consumes the message again.
//!
//! Other references:
//! 1. https://devcorner.digitalpress.blog/rabbitmq-retries-the-new-full-story/

use std::fmt::Debug;
use std::sync::Arc;

use amqprs::channel::*;
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver, FieldName, FieldTable, FieldValue};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use pqx::ec::util::*;
use pqx::ec::*;
use pqx::error::PqxResult;
use pqx::mq::*;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

// ================================================================================================
// const
// ================================================================================================

const EXCHG: &str = "rbmq-rs-exchange";
const EXCHG_DELAY: &str = "rbmq-rs-delay";
const ROUT: &str = "rbmq-rs-rout";
const QUE: &str = "rbmq-rs-que";

const CONDA_ENV: &str = "py310";

// ================================================================================================
// helper
// ================================================================================================

// PANIC if file not found!
fn get_conn_yaml_path() -> std::path::PathBuf {
    join_dir(current_dir().unwrap(), "conn.yml").unwrap()
}

// ================================================================================================
// DevMsg
// ================================================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DevMsg {
    cmd: CmdArg,
    time: DateTime<Local>,
}

impl DevMsg {
    fn new(cmd: CmdArg) -> Self {
        Self {
            cmd,
            time: Local::now(),
        }
    }
}

// ================================================================================================
// mock async functions for handling stdout & stderr
// ================================================================================================

#[instrument]
async fn logging_stdout(s: String) -> PqxResult<()> {
    info!("logging_stdout: {:?}", s);

    Ok(())
}

#[instrument]
async fn logging_stderr(s: String) -> PqxResult<()> {
    info!("logging_stderr: {:?}", s);

    Ok(())
}

// ================================================================================================
// impl PqxConsumer
// ================================================================================================

#[derive(Clone)]
struct CustomConsumer {
    max_retry: i16,
    poke: i32,
    original_rout: String,
    delayed_change: String,
    executor: CmdAsyncExecutor,
}

impl CustomConsumer {
    fn new(max_retry: i16, poke: i32, original_rout: &str, delayed_change: &str) -> Self {
        Self {
            max_retry,
            poke,
            original_rout: original_rout.to_owned(),
            delayed_change: delayed_change.to_owned(),
            executor: CmdAsyncExecutor::new(),
        }
    }
}

#[async_trait]
impl AsyncConsumer for CustomConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        mut basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let msg = if let Ok(m) = serde_json::from_slice::<DevMsg>(&content) {
            m
        } else {
            // if message cannot be recognized, then discard it
            let args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
            let _ = channel.basic_nack(args).await;
            return;
        };

        // execute command
        let res = self.executor.exec(1, msg.cmd).await;
        println!(">>> execute command: {:?}", res);

        match res {
            Ok(es) => {
                if es.success() {
                    let args = BasicAckArguments::new(deliver.delivery_tag(), false);
                    let _ = channel.basic_ack(args).await;
                } else {
                    // retry
                    let x_delay = FieldName::try_from(String::from("x-delay")).unwrap();
                    let x_retries = FieldName::try_from(String::from("x-retries")).unwrap();

                    let mut headers = basic_properties.headers().cloned().unwrap_or_else(|| {
                        let mut ft = FieldTable::new();
                        ft.insert(x_delay.clone(), FieldValue::I(self.poke));
                        ft.insert(x_retries.clone(), FieldValue::s(0));

                        ft
                    });

                    let retries = if let FieldValue::s(r) = headers.get(&x_retries).unwrap().clone()
                    {
                        r + 1
                    } else {
                        0
                    };
                    println!(">>> retries: {:?}", retries);
                    if retries >= self.max_retry {
                        // discard
                        let args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
                        println!(">>> discarding message, tried: {retries}");
                        let _ = channel.basic_nack(args).await;
                    } else {
                        // publish to `x-delayed` exchange
                        headers.remove(&x_retries);
                        headers.insert(x_retries, FieldValue::s(retries));

                        let props = basic_properties.with_headers(headers).finish();
                        let args =
                            BasicPublishArguments::new(&self.delayed_change, &self.original_rout);
                        println!(">>> republish to x-delayed exchange with retries: {retries}");
                        let _ = channel.basic_publish(props, content, args).await;
                        // consume this message
                        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
                        let _ = channel.basic_ack(args).await;
                        println!("basic_ack done");
                    }
                }
            }
            Err(_) => {
                // if execution failed, regard fault message, then discard it
                let args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
                let _ = channel.basic_nack(args).await;
            }
        }
    }
}

// ================================================================================================
// exchange & x-delayed-exchange & queue
// ================================================================================================

#[tokio::test]
async fn declare_exchanges_and_queue_success() {
    // 0. mq client
    let mut client = MqClient::new();
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 1. declare a normal exchange
    let res = client.declare_exchange(EXCHG, ExchangeType::Direct).await;
    assert!(res.is_ok());

    // 2. declare `x-delayed-message` exchange (needs plugin's support)
    let mut args = FieldTable::new();
    args.insert(
        FieldName::try_from("x-delayed-type").unwrap(),
        FieldValue::from(String::from("direct")),
    );
    let res = client
        .declare_exchange_with_args(
            EXCHG_DELAY,
            ExchangeType::Plugin(String::from("x-delayed-message")),
            args,
        )
        .await;
    assert!(res.is_ok());

    // 3. declare queue and bind to these exchanges
    let res = client.declare_queue(QUE).await;
    assert!(res.is_ok());
    let res = client.bind_queue(EXCHG, ROUT, QUE).await;
    assert!(res.is_ok());
    let res = client.bind_queue(EXCHG_DELAY, ROUT, QUE).await;
    assert!(res.is_ok());
}

// ================================================================================================
// subscriber
// ================================================================================================

#[tokio::test]
async fn mq_subscriber_success() {
    let file_appender = tracing_appender::rolling::daily("./log", "mq_subscriber.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    // 0. client and channel
    let mut client = MqClient::new();
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 1. custom consumer
    let mut consumer = CustomConsumer::new(5, 10000, ROUT, EXCHG_DELAY);
    consumer
        .executor
        .register_stdout_fn(Arc::new(logging_stdout));
    consumer
        .executor
        .register_stderr_fn(Arc::new(logging_stderr));

    // 2. new subscriber
    let mut subscriber = BasicSubscriber::new(client.channel().unwrap(), consumer);
    let res = subscriber.set_consumer_prefetch(0, 1, false).await;
    assert!(res.is_ok());

    // 3. consume
    let res = subscriber.consume(QUE).await;
    assert!(res.is_ok());

    println!("Start listening...");

    // 4. block
    subscriber.block().await;
}

// ================================================================================================
// publisher
// ================================================================================================

#[tokio::test]
async fn mq_publisher_success() {
    // 0. client and channel
    let mut client = MqClient::new();
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    //  1. new publisher
    let publisher = Publisher::new(client.channel().unwrap());

    // 2. message
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let cmd = CmdArg::conda_python(CONDA_ENV, dir.to_string_lossy(), "throw_error.py");
    // let cmd = CmdArg::conda_python(CONDA_ENV, dir.to_string_lossy(), "success.py");
    let msg = DevMsg::new(cmd);

    // 3. publish
    let res = publisher.publish(EXCHG, ROUT, msg).await;
    assert!(res.is_ok());

    // 4. block
    publisher.block(1).await;
}
