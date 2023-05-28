//! file: consumer.rs
//! author: Jacob Xie
//! date: 2023/05/28 11:06:07 Sunday
//! brief:

use amqprs::channel::{BasicAckArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;

use super::message::PqxMessage;

// ================================================================================================
// PqxConsumer
// ================================================================================================

#[derive(Clone)]
pub struct PqxDefaultConsumer {
    //
}

impl PqxDefaultConsumer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl AsyncConsumer for PqxDefaultConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        // deserialize from subscriber msg
        let msg: PqxMessage = match serde_json::from_slice(&content) {
            Ok(m) => m,
            Err(_) => PqxMessage::Undefined {
                msg: "undefined pqx message".to_string(),
            },
        };

        // TODO
        println!("msg: {:?}", msg);

        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        channel.basic_ack(args).await.unwrap();
    }
}
