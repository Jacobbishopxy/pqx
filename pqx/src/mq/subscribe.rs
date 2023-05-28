//! file: subscribe.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:55:05 Friday
//! brief:

use amqprs::channel::{BasicConsumeArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::BasicProperties;
use tokio::sync::Notify;

use crate::error::PqxResult;

// ================================================================================================
// Subscriber
// ================================================================================================

#[derive(Clone)]
pub struct Subscriber<'a, S>
where
    S: AsyncConsumer + Clone + Send + 'static,
{
    channel: &'a Channel,
    message_prop: BasicProperties,
    consumer: S,
}

impl<'a, S> Subscriber<'a, S>
where
    S: AsyncConsumer + Clone + Send + 'static,
{
    pub fn new(channel: &'a Channel, consumer: S) -> Self {
        Self {
            channel,
            message_prop: BasicProperties::default(),
            consumer,
        }
    }

    pub fn set_message_properties(&mut self, message_properties: BasicProperties) {
        self.message_prop = message_properties;
    }

    pub async fn consume(&self, que: &str, consumer_tag: &str) -> PqxResult<()> {
        let args = BasicConsumeArguments::new(que, consumer_tag);
        self.channel
            .basic_consume(self.consumer.clone(), args)
            .await?;

        Ok(())
    }

    pub async fn block(&self) {
        let guard = Notify::new();

        guard.notified().await;
    }
}
