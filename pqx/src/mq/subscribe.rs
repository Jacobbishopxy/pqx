//! file: subscribe.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:55:05 Friday
//!
//! Subscriber methods:
//! 1. set_consume_args
//! 2. set_consumer_prefetch
//! 3. set_consumer_priorities
//! 4. set_consumer_timeout
//! 5. consume
//! 6. cancel_consume
//! 7. block

use amqprs::channel::*;
use amqprs::consumer::AsyncConsumer;
use serde::de::DeserializeOwned;

use super::*;
use crate::error::PqxResult;

// ================================================================================================
// BasicSubscriber
// ================================================================================================

pub struct BasicSubscriber<'a, S>
where
    S: AsyncConsumer + Send + Clone + 'static,
{
    channel: &'a Channel,
    consume_args: Option<BasicConsumeArguments>,
    consumer: S,
    consumer_tag: Option<String>, // server generated tag, here we don't make it ourselves
}

impl<'a, S> BasicSubscriber<'a, S>
where
    S: AsyncConsumer + Send + Clone + 'static,
{
    pub fn new(channel: &'a Channel, consumer: S) -> Self {
        Self {
            channel,
            consume_args: Some(BasicConsumeArguments::default()),
            consumer,
            consumer_tag: None,
        }
    }

    impl_set_consume_args!();
    impl_set_consumer_prefetch!();
    impl_set_consumer_priorities!();
    impl_set_consumer_timeout!();
    impl_recover!();
    impl_consume!();
    impl_cancel_consume!();
    impl_block!();
}

// ================================================================================================
// Subscriber
// ================================================================================================

pub struct Subscriber<'a, M, S>
where
    M: Send + Sync + DeserializeOwned + 'static,
    S: Send + Sync + Consumer<M> + 'static,
{
    channel: &'a Channel,
    consume_args: Option<BasicConsumeArguments>,
    consumer: ConsumerWrapper<M, S>,
    consumer_tag: Option<String>,
    queue: Option<String>,
}

impl<'a, M, S> Subscriber<'a, M, S>
where
    M: Send + Sync + DeserializeOwned + Clone + 'static,
    S: Send + Sync + Consumer<M> + 'static,
{
    pub fn new(channel: &'a Channel, consumer: S) -> Self {
        Self {
            channel,
            consume_args: Some(BasicConsumeArguments::default()),
            consumer: ConsumerWrapper::new(consumer),
            consumer_tag: None,
            queue: None,
        }
    }

    impl_set_consume_args!();
    impl_set_consumer_prefetch!();
    impl_set_consumer_priorities!();
    impl_set_consumer_timeout!();
    impl_recover!();
    impl_block!();

    pub async fn consume(&mut self, que: &str) -> PqxResult<()> {
        let consumer = self.consumer.clone();

        let args = self
            .consume_args
            .take()
            .unwrap()
            .queue(que.to_owned())
            .finish();

        // start to consume
        self.consumer.signal_consume(true).await;
        let consumer_tag = self.channel.basic_consume(consumer, args).await?;
        // save consumer tag
        self.consumer_tag = Some(consumer_tag);
        self.queue = Some(que.to_owned());

        Ok(())
    }

    pub async fn cancel_consume(&mut self, no_wait: bool) -> PqxResult<()> {
        let consumer_tag = if let Some(ct) = self.consumer_tag.take() {
            ct
        } else {
            return Err("consumer tag is empty, check whether consuming starts".into());
        };
        let args = BasicCancelArguments {
            consumer_tag,
            no_wait,
        };

        self.consumer.signal_consume(false).await;
        self.channel.basic_cancel(args).await?;

        Ok(())
    }

    pub async fn resume_consume(&mut self) -> PqxResult<()> {
        let q = match self.queue.as_ref() {
            Some(q) => q,
            None => {
                return Err("haven't started consume yet".into());
            }
        }
        .clone();
        self.consumer.signal_consume(true).await;
        self.consume(&q).await?;

        Ok(())
    }

    pub async fn soft_fail_block(&mut self) {
        let rx = self.consumer.consume_signal_receiver();

        while let Some(sig) = rx.lock().await.recv().await {
            if !sig {
                break;
            }
        }
    }
}
