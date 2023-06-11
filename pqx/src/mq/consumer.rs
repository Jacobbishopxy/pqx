//! file: consumer.rs
//! author: Jacob Xie
//! date: 2023/05/28 11:06:07 Sunday
//! brief:

use std::marker::PhantomData;

use amqprs::channel::{BasicAckArguments, BasicNackArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::PqxResult;

// ================================================================================================
// PqxConsumer
// ================================================================================================

#[derive(Clone)]
pub struct PqxDefaultConsumer;

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
        match serde_json::from_slice::<Value>(&content) {
            Ok(m) => println!("msg: {:?}", m),
            Err(e) => println!("err: {:?}", e),
        };

        // according to biz logic determine which following method to be called: ack/nack/reject/recover

        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        channel.basic_ack(args).await.unwrap();
    }
}

// ================================================================================================
// Consumer
// ================================================================================================

#[async_trait]
pub trait Consumer<M: DeserializeOwned>: Clone {
    // consumer behavior:
    // Ok(true) => handle_success
    // Ok(false) => handle_requeue
    // Err(_) => handle_discard
    async fn consume(&mut self, content: M) -> PqxResult<bool>;

    // ================================================================================================
    // default implementation
    //
    // the following methods can be overridden
    // ================================================================================================

    // TODO: message as input param

    #[allow(unused_variables)]
    fn handle_props(&self, props: BasicProperties) {}

    async fn success_callback(&mut self) -> PqxResult<()> {
        Ok(())
    }
    async fn requeue_callback(&mut self) -> PqxResult<()> {
        Ok(())
    }
    async fn discard_callback(&mut self) -> PqxResult<()> {
        Ok(())
    }
}

// ================================================================================================
// ConsumerWrapper<T>
// A generic type holder for impl AsyncConsumer
//
// User doesn't need this struct, since it is a holder of user biz logic
// ================================================================================================

#[derive(Clone)]
pub(crate) struct ConsumerWrapper<M, T>
where
    M: Send + DeserializeOwned,
    T: Send + Consumer<M>,
{
    consumer: T,
    should_cancel_consumer: bool,
    _msg_type: PhantomData<M>,
}

impl<M, T> ConsumerWrapper<M, T>
where
    M: Send + DeserializeOwned,
    T: Send + Consumer<M>,
{
    pub fn new(consumer: T) -> Self {
        Self {
            consumer,
            should_cancel_consumer: false,
            _msg_type: PhantomData,
        }
    }

    pub fn consumer(&mut self) -> &mut T {
        &mut self.consumer
    }

    // ================================================================================================
    // private methods
    // ================================================================================================

    async fn ack<'a>(&'a mut self, channel: &'a Channel, deliver: Deliver) -> PqxResult<()> {
        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        channel.basic_ack(args).await?;

        Ok(())
    }

    async fn nack<'a>(
        &'a mut self,
        channel: &'a Channel,
        deliver: Deliver,
        requeue: bool,
    ) -> PqxResult<()> {
        let args = BasicNackArguments::new(deliver.delivery_tag(), false, requeue);
        channel.basic_nack(args).await?;

        Ok(())
    }

    async fn handle_success(&mut self, channel: &Channel, deliver: Deliver) {
        // if callback failed, set `cancel_consumer` to true
        if let Err(_) = self.consumer().success_callback().await {
            self.should_cancel_consumer = true;
        };
        let _ = self.ack(channel, deliver).await;
    }

    async fn handle_requeue(&mut self, channel: &Channel, deliver: Deliver) {
        // if callback failed, set `cancel_consumer` to true
        if let Err(_) = self.consumer().requeue_callback().await {
            self.should_cancel_consumer = true;
        };
        let _ = self.nack(channel, deliver, true).await;
    }

    async fn handle_discard(&mut self, channel: &Channel, deliver: Deliver) {
        // if callback failed, set `cancel_consumer` to true
        if let Err(_) = self.consumer().discard_callback().await {
            self.should_cancel_consumer = true;
        };
        let _ = self.nack(channel, deliver, false).await;
    }
}

#[async_trait]
impl<M, T> AsyncConsumer for ConsumerWrapper<M, T>
where
    M: Send + Sync + DeserializeOwned,
    T: Send + Sync + Consumer<M>,
{
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        // deserialize from subscriber msg. simply discard message if cannot be deserialize
        let msg = match serde_json::from_slice::<M>(&content) {
            Ok(m) => m,
            Err(_) => {
                self.handle_discard(channel, deliver).await;
                return;
            }
        };

        // handle props
        self.consumer().handle_props(basic_properties);
        // future result
        let fut_res = self.consumer().consume(msg).await;

        // according to biz logic determine whether responds Ack/Requque/Discard
        match fut_res {
            Ok(true) => self.handle_success(channel, deliver).await,
            Ok(false) => self.handle_requeue(channel, deliver).await,
            Err(_) => self.handle_discard(channel, deliver).await,
        };
    }
}
