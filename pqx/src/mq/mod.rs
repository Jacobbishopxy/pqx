//! file: mod.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:52:32 Friday
//! brief:

pub mod client;
pub mod consumer;
pub mod predefined;
pub mod publish;
pub mod subscribe;

pub use client::*;
pub use consumer::*;
pub use predefined::*;
pub use publish::*;
pub use subscribe::*;

// ================================================================================================
// private macros
// ================================================================================================

macro_rules! get_connection {
    ($s:ident) => {
        $s.connection.as_ref().ok_or("connection is empty")
    };
}

macro_rules! get_channel {
    ($s:ident) => {
        $s.channel.as_ref().ok_or("channel is empty")
    };
}

pub(crate) use get_channel;
pub(crate) use get_connection;

///////////////////////////////////////////////////////////////////////////////////////////////////

macro_rules! impl_set_consume_args {
    () => {
        pub fn set_consume_args(&mut self, consume_args: ::amqprs::channel::BasicConsumeArguments) {
            self.consume_args = Some(consume_args);
        }
    };
}

macro_rules! impl_set_consumer_prefetch {
    () => {
        pub async fn set_consumer_prefetch(
            &self,
            size: u32,
            count: u16,
            global: bool,
        ) -> crate::error::PqxResult<()> {
            let args = ::amqprs::channel::BasicQosArguments::new(size, count, global);

            self.channel.basic_qos(args).await?;

            Ok(())
        }
    };
}

macro_rules! impl_set_consumer_priorities {
    () => {
        pub async fn set_consumer_priorities(
            &mut self,
            priority: i16,
        ) -> crate::error::PqxResult<()> {
            let x_priority = ::amqprs::FieldName::try_from("x-priority").unwrap();

            // no matter whether "x-priority" exists, remove it and insert a new one
            self.consume_args
                .as_mut()
                .unwrap()
                .arguments
                .remove(&x_priority);

            self.consume_args
                .as_mut()
                .unwrap()
                .arguments
                .insert(x_priority, ::amqprs::FieldValue::s(priority));

            Ok(())
        }
    };
}

macro_rules! impl_set_consumer_timeout {
    () => {
        pub async fn set_consumer_timeout(&mut self, timeout: u32) -> crate::error::PqxResult<()> {
            let x_consumer_timeout = ::amqprs::FieldName::try_from("x-consumer-timeout").unwrap();

            self.consume_args
                .as_mut()
                .unwrap()
                .arguments
                .remove(&x_consumer_timeout);

            self.consume_args
                .as_mut()
                .unwrap()
                .arguments
                .insert(x_consumer_timeout, ::amqprs::FieldValue::i(timeout));

            Ok(())
        }
    };
}

macro_rules! impl_recover {
    () => {
        pub async fn recover(&self, requeue: bool) -> crate::error::PqxResult<()> {
            self.channel.basic_recover(requeue).await?;

            Ok(())
        }
    };
}

macro_rules! impl_consume {
    () => {
        pub async fn consume(&mut self, que: &str) -> crate::error::PqxResult<()> {
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
    };
}

macro_rules! impl_cancel_consume {
    () => {
        pub async fn cancel_consume(&mut self, no_wait: bool) -> crate::error::PqxResult<()> {
            let consumer_tag = if let Some(ct) = self.consumer_tag.take() {
                ct
            } else {
                return Err("consumer tag is empty, check whether consuming starts".into());
            };
            let args = ::amqprs::channel::BasicCancelArguments {
                consumer_tag,
                no_wait,
            };

            self.channel.basic_cancel(args).await?;

            Ok(())
        }
    };
}

macro_rules! impl_block {
    () => {
        pub async fn block(&self) {
            let guard = ::tokio::sync::Notify::new();

            guard.notified().await;
        }
    };
}

pub(crate) use impl_block;
pub(crate) use impl_cancel_consume;
pub(crate) use impl_consume;
pub(crate) use impl_recover;
pub(crate) use impl_set_consume_args;
pub(crate) use impl_set_consumer_prefetch;
pub(crate) use impl_set_consumer_priorities;
pub(crate) use impl_set_consumer_timeout;

///////////////////////////////////////////////////////////////////////////////////////////////////
