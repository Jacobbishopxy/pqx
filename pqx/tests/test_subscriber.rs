//! file: test_subscriber.rs
//! author: Jacob Xie
//! date: 2023/05/28 21:38:07 Sunday
//! brief:

use std::sync::Arc;

use amqprs::channel::{BasicAckArguments, Channel, ExchangeType};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use pqx::ec::util::*;
use pqx::ec::*;
use pqx::error::PqxResult;
use pqx::mq::*;

// ================================================================================================
// const
// ================================================================================================

const EXCHG: &str = "rbmq-rs-exchange";
const ROUT: &str = "rbmq-rs-rout";
const QUE: &str = "rbmq-rs-que";

// ================================================================================================
// helper
// ================================================================================================

// PANIC if file not found!
fn get_conn_yaml_path() -> std::path::PathBuf {
    join_dir(current_dir().unwrap(), "conn.yml").unwrap()
}

// ================================================================================================
// impl AsyncConsumer
// ================================================================================================

#[derive(Clone)]
struct DevCmdConsumer {
    executor: CmdAsyncExecutor,
}

impl DevCmdConsumer {
    fn new() -> Self {
        Self {
            executor: CmdAsyncExecutor::new(),
        }
    }
}

#[async_trait]
impl AsyncConsumer for DevCmdConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        // deserialize from subscriber msg
        match serde_json::from_slice::<CmdArg>(&content) {
            Ok(m) => {
                println!("msg: {:?}", m);

                let res = self.executor.exec(1, m).await;
                println!("res: {:?}", res);
            }
            Err(e) => println!("err: {:?}", e),
        };

        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        channel.basic_ack(args).await.unwrap();
    }
}

// ================================================================================================
// mock async functions
// ================================================================================================

async fn a_print_stdout(s: String) -> PqxResult<()> {
    println!("print_stdout: {:?}", s);

    Ok(())
}

async fn a_print_stderr(s: String) -> PqxResult<()> {
    println!("print_stderr: {:?}", s);

    Ok(())
}

// ================================================================================================
// subscriber (same as subscriber in `test_mq.rs`, except use custom consumer)
// ================================================================================================

#[tokio::test]
async fn mq_subscribe_success() {
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

    // 4. [IMPORTANT!!!] new consumer: cmd executor
    let mut consumer = DevCmdConsumer::new();
    consumer
        .executor
        .register_stdout_fn(Arc::new(a_print_stdout));
    consumer
        .executor
        .register_stderr_fn(Arc::new(a_print_stderr));

    // 5. new subscriber
    let mut subscriber = Subscriber::new(client.channel().unwrap(), consumer);

    // 6. consume
    let res = subscriber.consume(QUE).await;
    assert!(res.is_ok());

    println!("Start listening on {}:{} ...", "HOST", "PORT");

    // 7. block
    subscriber.block().await;
}

// ================================================================================================
// publisher (same as publisher in `test_mq.rs`)
// ================================================================================================

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

    // 4. prepare message to be sent
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let msg = CmdArg::CondaPython {
        env: "py38".to_owned(),
        dir: dir.to_str().unwrap().to_string(),
        script: "print_csv_in_line.py".to_string(),
    };

    // 5. send to RabbitMq
    let res = publisher.publish(EXCHG, ROUT, msg).await;
    assert!(res.is_ok());

    // 6. block a second to wait publish done
    publisher.block(1).await
}
