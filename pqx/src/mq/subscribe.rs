//! file: subscribe.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:55:05 Friday
//! brief:

use amqprs::channel::BasicConsumeArguments;
use amqprs::consumer::AsyncConsumer;
use amqprs::BasicProperties;
use tokio::sync::Notify;

use super::client::MqClient;
use crate::error::PqxResult;

// ================================================================================================
// Subscriber
// ================================================================================================

#[derive(Clone)]
pub struct Subscriber<C, S>
where
    C: AsRef<MqClient>,
    S: AsyncConsumer + Clone + Send + 'static,
{
    client: C,
    consumer: S,
    message_prop: BasicProperties,
}

impl<C, S> Subscriber<C, S>
where
    C: AsRef<MqClient>,
    S: AsyncConsumer + Clone + Send + 'static,
{
    pub fn new(client: C, consumer: S) -> Self {
        Self {
            client,
            consumer,
            message_prop: BasicProperties::default(),
        }
    }

    pub fn set_message_properties(&mut self, message_properties: BasicProperties) {
        self.message_prop = message_properties;
    }

    pub async fn consume(&self, que: &str, consumer_tag: &str) -> PqxResult<()> {
        let args = BasicConsumeArguments::new(que, consumer_tag);
        self.client
            .as_ref()
            .channel()?
            .basic_consume(self.consumer.clone(), args)
            .await?;

        Ok(())
    }

    pub async fn block(&self) {
        let guard = Notify::new();

        guard.notified().await;
    }
}
