//! file: test_mq.rs
//! author: Jacob Xie
//! date: 2023/05/28 12:34:16 Sunday
//! brief:

use once_cell::sync::Lazy;
use pqx::ec::cmd::CmdArg;
use pqx::ec::util::*;
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

static CONN_ARG: Lazy<ConnArg> = Lazy::new(|| ConnArg {
    host: HOST,
    port: PORT,
    user: USER,
    pass: PASS,
    vhost: Some(VHOST),
});

// ================================================================================================
// subscriber
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
    let chan = client.channel().unwrap();

    // 3. declare queue
    let res = client.declare_and_bind_queue(EXCHG, ROUT, QUE).await;
    assert!(res.is_ok());

    // 4. new consumer
    let consumer = PqxDefaultConsumer;

    // 5. new subscriber
    let mut subscriber = Subscriber::new(chan, consumer);

    // 6. consume
    let res = subscriber.consume(QUE).await;
    assert!(res.is_ok());

    println!("Start listening on {}:{} ...", HOST, PORT);

    // 7. block
    subscriber.block().await;
}

// ================================================================================================
// publisher
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
    let chan = client.channel().unwrap();

    // 3. new publisher
    let publisher = Publisher::new(chan);

    // 4. prepare message to be sent
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let msg = CmdArg::conda_python("py310", dir.to_str().unwrap(), "print_csv_in_line.py");

    // 5. send to RabbitMq
    let res = publisher.publish(EXCHG, ROUT, msg).await;
    assert!(res.is_ok());

    // 6. block a second to wait publish done
    publisher.block(1).await
}
