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
    impl_consume!();
    impl_cancel_consume!();
    impl_block!();
}

// ================================================================================================
// Subscriber
// ================================================================================================

pub struct Subscriber<'a, M, T>
where
    M: Send + Sync + DeserializeOwned + 'static,
    T: Send + Sync + Consumer<M> + 'static,
{
    channel: &'a Channel,
    consume_args: Option<BasicConsumeArguments>,
    consumer: ConsumerWrapper<M, T>,
    consumer_tag: Option<String>,
}

impl<'a, M, T> Subscriber<'a, M, T>
where
    M: Send + Sync + DeserializeOwned + Clone + 'static,
    T: Send + Sync + Consumer<M> + 'static,
{
    pub fn new(channel: &'a Channel, consumer: T) -> Self {
        Self {
            channel,
            consume_args: Some(BasicConsumeArguments::default()),
            consumer: ConsumerWrapper::new(consumer),
            consumer_tag: None,
        }
    }

    impl_set_consume_args!();
    impl_set_consumer_prefetch!();
    impl_set_consumer_priorities!();
    impl_set_consumer_timeout!();
    impl_consume!();
    impl_cancel_consume!();
    impl_block!();
}
