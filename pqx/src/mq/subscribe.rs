//! file: subscribe.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:55:05 Friday
//! brief:

use std::marker::PhantomData;

use amqprs::channel::*;
use amqprs::consumer::AsyncConsumer;
use amqprs::{FieldName, FieldValue};
use serde::de::DeserializeOwned;
use tokio::sync::Notify;

use crate::error::PqxResult;

use super::{ConsumerT, ConsumerWrapper};

// ================================================================================================
// BasicSubscriber
//
// methods:
// 1. set_consume_args
// 2. set_consumer_prefetch
// 3. set_consumer_priorities
// 4. consume
// 5. cancel_consume
// 6. block
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

    pub fn set_consume_args(&mut self, consume_args: BasicConsumeArguments) {
        self.consume_args = Some(consume_args);
    }

    pub async fn set_consumer_prefetch(
        &self,
        size: u32,
        count: u16,
        global: bool,
    ) -> PqxResult<()> {
        let args = BasicQosArguments::new(size, count, global);

        self.channel.basic_qos(args).await?;

        Ok(())
    }

    pub async fn set_consumer_priorities(&mut self, priority: i16) -> PqxResult<()> {
        let x_priority = FieldName::try_from("x-priority").unwrap();

        // no matter whether "x-priority" exists, remove it and insert a new one
        self.consume_args
            .as_mut()
            .unwrap()
            .arguments
            .remove(&x_priority);

        self.consume_args.as_mut().unwrap().arguments.insert(
            FieldName::try_from("x-priority").unwrap(),
            FieldValue::s(priority),
        );

        Ok(())
    }

    pub async fn consume(&mut self, que: &str) -> PqxResult<()> {
        let consumer = self.consumer.clone();

        let args = self
            .consume_args
            .take()
            .unwrap()
            .queue(que.to_owned())
            .finish();

        // start to consume
        let consumer_tag = self.channel.basic_consume(consumer, args).await?;
        // save consumer tag
        self.consumer_tag = Some(consumer_tag);

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

        self.channel.basic_cancel(args).await?;

        Ok(())
    }

    pub async fn block(&self) {
        let guard = Notify::new();

        guard.notified().await;
    }
}

// ================================================================================================
// Subscriber
// ================================================================================================

pub struct Subscriber<'a, M, T>
where
    M: Send + Sync + DeserializeOwned + 'static,
    T: Send + Sync + ConsumerT<M> + 'static,
{
    channel: &'a Channel,
    consume_args: Option<BasicConsumeArguments>,
    consumer: ConsumerWrapper<M, T>,
    consumer_tag: Option<String>,
    _msg_type: PhantomData<M>,
}

impl<'a, M, T> Subscriber<'a, M, T>
where
    M: Send + Sync + DeserializeOwned + Clone + 'static,
    T: Send + Sync + ConsumerT<M> + 'static,
{
    pub fn new(channel: &'a Channel, consumer: T) -> Self {
        Self {
            channel,
            consume_args: Some(BasicConsumeArguments::default()),
            consumer: ConsumerWrapper::new(consumer),
            consumer_tag: None,
            _msg_type: PhantomData,
        }
    }

    pub fn set_consume_args(&mut self, consume_args: BasicConsumeArguments) {
        self.consume_args = Some(consume_args);
    }

    pub async fn set_consumer_prefetch(
        &self,
        size: u32,
        count: u16,
        global: bool,
    ) -> PqxResult<()> {
        let args = BasicQosArguments::new(size, count, global);

        self.channel.basic_qos(args).await?;

        Ok(())
    }

    pub async fn set_consumer_priorities(&mut self, priority: i16) -> PqxResult<()> {
        let x_priority = FieldName::try_from("x-priority").unwrap();

        // no matter whether "x-priority" exists, remove it and insert a new one
        self.consume_args
            .as_mut()
            .unwrap()
            .arguments
            .remove(&x_priority);

        self.consume_args.as_mut().unwrap().arguments.insert(
            FieldName::try_from("x-priority").unwrap(),
            FieldValue::s(priority),
        );

        Ok(())
    }

    pub async fn consume(&mut self, que: &str) -> PqxResult<()> {
        let consumer = self.consumer.clone();

        let args = self
            .consume_args
            .take()
            .unwrap()
            .queue(que.to_owned())
            .finish();

        // start to consume
        let consumer_tag = self.channel.basic_consume(consumer, args).await?;
        // save consumer tag
        self.consumer_tag = Some(consumer_tag);

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

        self.channel.basic_cancel(args).await?;

        Ok(())
    }

    pub async fn block(&self) {
        let guard = Notify::new();

        guard.notified().await;
    }
}
