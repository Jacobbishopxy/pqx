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
    S: AsyncConsumer + Send + 'static,
{
    channel: &'a Channel,
    message_prop: BasicProperties,
    consumer: Option<S>,
}

impl<'a, S> Subscriber<'a, S>
where
    S: AsyncConsumer + Send + 'static,
{
    pub fn new(channel: &'a Channel, consumer: S) -> Self {
        Self {
            channel,
            message_prop: BasicProperties::default(),
            consumer: Some(consumer),
        }
    }

    pub fn set_message_properties(&mut self, message_properties: BasicProperties) {
        self.message_prop = message_properties;
    }

    pub async fn consume(&mut self, que: &str) -> PqxResult<()> {
        let consumer = if let Some(c) = self.consumer.take() {
            c
        } else {
            return Err("consumer is empty".into());
        };
        let args = BasicConsumeArguments::new(que, "");

        self.channel.basic_consume(consumer, args).await?;

        Ok(())
    }

    pub async fn block(&self) {
        let guard = Notify::new();

        guard.notified().await;
    }
}
