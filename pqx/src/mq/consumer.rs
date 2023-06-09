//! file: consumer.rs
//! author: Jacob Xie
//! date: 2023/05/28 11:06:07 Sunday
//! brief:

use std::marker::PhantomData;

use amqprs::channel::{BasicAckArguments, BasicNackArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use futures::future::BoxFuture;
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

pub trait Consumer<M: DeserializeOwned>: Clone {
    fn consume<'a>(&'a mut self, content: M) -> BoxFuture<'a, PqxResult<bool>>;

    // default impl
    #[allow(unused_variables)]
    fn handle_props(&self, props: BasicProperties) {}
}

// ================================================================================================
// ConsumerWrapper<T>
// A generic type holder for impl AsyncConsumer
// ================================================================================================

#[derive(Clone)]
pub struct ConsumerWrapper<M, T>
where
    M: Send + DeserializeOwned,
    T: Send + Consumer<M>,
{
    consumer: T,
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
            _msg_type: PhantomData,
        }
    }

    pub fn consumer(&mut self) -> &mut T {
        &mut self.consumer
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
                let args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
                // ignore error
                let _ = channel.basic_nack(args).await;
                return;
            }
        };

        // handle props
        self.consumer().handle_props(basic_properties);
        // future result
        let fut_res = self.consumer().consume(msg).await;

        // according to biz logic determine whether responds Ack/Requque/Discard
        match fut_res {
            Ok(true) => {
                let args = BasicAckArguments::new(deliver.delivery_tag(), false);
                let _ = channel.basic_ack(args).await;
            }
            Ok(false) => {
                // requeue == true
                let args = BasicNackArguments::new(deliver.delivery_tag(), false, true);
                let _ = channel.basic_nack(args).await;
            }
            Err(_) => {
                // requeue == false
                let args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
                // ignore error
                let _ = channel.basic_nack(args).await;
            }
        }
    }
}
