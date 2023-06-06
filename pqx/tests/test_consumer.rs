//! file: test_consumer.rs
//! author: Jacob Xie
//! date: 2023/06/06 17:36:11 Tuesday
//! brief:

use amqprs::channel::{BasicAckArguments, Channel, ExchangeType};
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use futures::future::BoxFuture;
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
// impl PqxConsumer
// ================================================================================================

struct CustomConsumer;

impl<'a> ConsumerT<'a, String> for CustomConsumer {
    fn consume(&mut self, content: String) -> BoxFuture<'a, PqxResult<bool>> {
        let fut = async move {
            print!("received content: {:?}", content);

            Ok(true)
        };

        Box::pin(fut)
    }
}

// ================================================================================================
// subscriber
// ================================================================================================
