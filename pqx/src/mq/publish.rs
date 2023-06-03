//! file: publish.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:54:55 Friday
//! brief:

use amqprs::channel::{BasicPublishArguments, Channel};
use amqprs::BasicProperties;
use serde::Serialize;

use crate::error::PqxResult;

// ================================================================================================
// Publisher
// ================================================================================================

#[derive(Clone)]
pub struct Publisher<'a> {
    channel: &'a Channel,
    message_prop: BasicProperties,
}

impl<'a> Publisher<'a> {
    pub fn new(channel: &'a Channel) -> Self {
        Self {
            channel,
            message_prop: BasicProperties::default(),
        }
    }

    pub fn set_message_properties(&mut self, message_properties: BasicProperties) {
        self.message_prop = message_properties;
    }

    pub async fn publish<M>(&self, exchange: &str, rout: &str, msg: M) -> PqxResult<()>
    where
        M: Serialize,
    {
        let args = BasicPublishArguments::new(exchange, rout);
        let content = serde_json::to_vec(&msg)?;
        self.channel
            .basic_publish(self.message_prop.clone(), content, args)
            .await?;

        Ok(())
    }

    pub async fn publish_with_props<M>(
        &self,
        exchange: &str,
        rout: &str,
        msg: M,
        props: BasicProperties,
    ) -> PqxResult<()>
    where
        M: Serialize,
    {
        let args = BasicPublishArguments::new(exchange, rout);
        let content = serde_json::to_vec(&msg)?;
        self.channel.basic_publish(props, content, args).await?;

        Ok(())
    }

    pub async fn block(&self, secs: u64) {
        tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
    }
}
