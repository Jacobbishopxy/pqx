//! file: test_callback.rs
//! author: Jacob Xie
//! date: 2023/06/08 23:09:22 Thursday
//! brief:

use amqprs::callbacks::{ChannelCallback, ConnectionCallback};
use amqprs::channel::Channel;
use amqprs::connection::Connection;
use amqprs::{Ack, BasicProperties, Cancel, Close, CloseChannel, Nack, Return};
use async_trait::async_trait;
use pqx::ec::util::*;
use pqx::mq::MqClient;

type Result<T> = std::result::Result<T, amqprs::error::Error>;

// ================================================================================================
// const
// ================================================================================================

// const EXCHG: &str = "rbmq-rs-exchange";
// const ROUT: &str = "rbmq-rs-rout";
// const QUE: &str = "rbmq-rs-que";

// ================================================================================================
// helper
// ================================================================================================

// PANIC if file not found!
fn get_conn_yaml_path() -> std::path::PathBuf {
    join_dir(current_dir().unwrap(), "conn.yml").unwrap()
}

// ================================================================================================
// ConnectionCallback
// ================================================================================================

struct ConnC;

#[allow(unused_variables)]
#[async_trait]
impl ConnectionCallback for ConnC {
    async fn close(&mut self, connection: &Connection, close: Close) -> Result<()> {
        Ok(())
    }

    async fn blocked(&mut self, connection: &Connection, reason: String) {}

    async fn unblocked(&mut self, connection: &Connection) {}
}

// ================================================================================================
// ChannelCallback
// ================================================================================================

struct ChanC;

#[allow(unused_variables)]
#[async_trait]
impl ChannelCallback for ChanC {
    async fn close(&mut self, channel: &Channel, close: CloseChannel) -> Result<()> {
        Ok(())
    }

    async fn cancel(&mut self, channel: &Channel, cancel: Cancel) -> Result<()> {
        Ok(())
    }

    async fn flow(&mut self, channel: &Channel, active: bool) -> Result<bool> {
        Ok(true)
    }

    async fn publish_ack(&mut self, channel: &Channel, ack: Ack) {}

    async fn publish_nack(&mut self, channel: &Channel, nack: Nack) {}

    async fn publish_return(
        &mut self,
        channel: &Channel,
        ret: Return,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
    }
}

// ================================================================================================
// test
// ================================================================================================

#[tokio::test]
async fn register_callback_success() {
    // 0. create mq client
    let mut client = MqClient::new();

    // 1. connect to RabbitMQ, and register connection callback
    let pth = get_conn_yaml_path();
    let res = client.connect_by_yaml(pth.to_str().unwrap()).await;
    assert!(res.is_ok());
    let res = client.connection().unwrap().register_callback(ConnC).await;
    assert!(res.is_ok());

    // 2. open channel, and register channel callback
    let res = client.open_channel(None).await;
    assert!(res.is_ok());
    let res = client.channel().unwrap().register_callback(ChanC).await;
    assert!(res.is_ok());
}
