//! file: test_consumer.rs
//! author: Jacob Xie
//! date: 2023/06/06 17:36:11 Tuesday
//! brief:

use std::fmt::Debug;
use std::sync::Arc;

use amqprs::channel::ExchangeType;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use pqx::ec::util::*;
use pqx::ec::CmdArg;
use pqx::ec::CmdAsyncExecutor;
use pqx::error::PqxResult;
use pqx::mq::*;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

// ================================================================================================
// const
// ================================================================================================

const EXCHG: &str = "rbmq-rs-exchange";
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
// impl PqxConsumer
// ================================================================================================

#[derive(Clone)]
struct CustomConsumer {
    executor: CmdAsyncExecutor,
}

impl Debug for CustomConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomConsumer")
            .field("executor", &"CmdAsyncExecutor")
            .finish()
    }
}

impl CustomConsumer {
    fn new() -> Self {
        Self {
            executor: CmdAsyncExecutor::new(),
        }
    }
}

#[async_trait]
impl Consumer<DevMsg> for CustomConsumer {
    #[instrument]
    async fn consume(&mut self, content: DevMsg) -> PqxResult<bool> {
        let es = self.executor.exec(1, content.cmd).await?;

        Ok(es.success())
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
// test
// ================================================================================================

#[tokio::test]
async fn mq_subscriber_success() {
    let file_appender = tracing_appender::rolling::daily("./log", "mq_subscriber.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    // 0. create mq client
    let mut client = MqClient::new();

    // 1. connect to RabbitMq
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());

    // 2. open channel
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 3. declare exchange & queue
    let res = client.declare_exchange(EXCHG, ExchangeType::Direct).await;
    assert!(res.is_ok());
    let res = client.declare_and_bind_queue(EXCHG, ROUT, QUE).await;
    assert!(res.is_ok());

    // 4. custom customer
    let mut consumer = CustomConsumer::new();
    consumer
        .executor
        .register_stdout_fn(Arc::new(logging_stdout));
    consumer
        .executor
        .register_stderr_fn(Arc::new(logging_stderr));

    // 5. new subscriber, and set `prefetch` to 1
    let mut subscriber = Subscriber::new(client.channel().unwrap(), consumer);
    let res = subscriber.set_consumer_prefetch(0, 1, false).await;
    assert!(res.is_ok());

    // 6. consume
    let res = subscriber.consume(QUE).await;
    assert!(res.is_ok());

    println!("Start listening...");

    // 7. block
    subscriber.block().await;
}

#[tokio::test]
async fn mq_publish_success() {
    // 0. create mq client
    let mut client = MqClient::new();

    // 1. connect to RabbitMQ
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());

    // 2. open channel
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 3. new publisher
    let publisher = Publisher::new(client.channel().unwrap());

    // 4. message
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let cmd = CmdArg::conda_python(CONDA_ENV, dir.to_string_lossy(), "print_csv_in_line.py");
    let msg = DevMsg::new(cmd);

    // 5. send
    let res = publisher.publish(EXCHG, ROUT, msg).await;
    assert!(res.is_ok());

    // 6. block 1 sec
    publisher.block(1).await;
}
