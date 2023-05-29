//! file: test_cmd_subscriber.rs
//! author: Jacob Xie
//! date: 2023/05/28 21:38:07 Sunday
//! brief:

use std::sync::Arc;

use amqprs::channel::{BasicAckArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use pqx::ec::util::*;
use pqx::ec::*;
use pqx::error::PqxResult;
use pqx::mq::*;

// ================================================================================================
// const
// ================================================================================================

const HOST: &str = "localhost";
const PORT: u16 = 5672;
const USER: &str = "dev";
const PASS: &str = "devpass";
const VHOST: &str = "devhost";
const EXCHG: &str = "amq.direct";
const ROUT: &str = "rbmq-rs-rout";
const QUE: &str = "rbmq-rs-que";
const TAG: &str = "rbmq-rs-tag";

static CONN_ARG: Lazy<ConnArg> = Lazy::new(|| ConnArg {
    host: HOST,
    port: PORT,
    user: USER,
    pass: PASS,
    vhost: Some(VHOST),
});

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
    let conn_arg = CONN_ARG.clone();

    let mut client = MqClient::new();
    // 1. connect to RabbitMq
    let res = client.connect(conn_arg).await;
    assert!(res.is_ok());

    // 2. open channel
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 3. declare queue
    let res = client.declare_queue(EXCHG, ROUT, QUE).await;
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
    let subscriber = Subscriber::new(client.channel().unwrap(), consumer);

    // 6. consume
    let res = subscriber.consume(QUE, TAG).await;
    assert!(res.is_ok());

    println!("Start listening on {}:{} ...", HOST, PORT);

    // 7. block
    subscriber.block().await;
}

// ================================================================================================
// publisher (same as publisher in `test_mq.rs`)
// ================================================================================================

#[tokio::test]
async fn mq_publish_success() {
    let conn_arg = CONN_ARG.clone();

    let mut client = MqClient::new();
    // 1. connect to RabbitMQ
    let res = client.connect(conn_arg).await;
    assert!(res.is_ok());

    // 2. open channel
    let res = client.open_channel(None).await;
    assert!(res.is_ok());

    // 3. new publisher
    let publisher = Publisher::new(client.channel().unwrap());

    // 4. prepare message to be sent
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let msg = CmdArg::CondaPython {
        env: "py310",
        dir: dir.to_str().unwrap(),
        script: "print_csv_in_line.py",
    };

    // 5. send to RabbitMq
    let res = publisher.publish(EXCHG, ROUT, msg).await;
    assert!(res.is_ok());

    // 6. block a second to wait publish done
    publisher.block(1).await
}
