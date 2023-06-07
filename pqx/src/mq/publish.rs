//! file: publish.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:54:55 Friday
//! brief:

use amqprs::channel::{BasicPublishArguments, Channel};
use amqprs::{BasicProperties, FieldTable};
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

    pub fn set_message_header(&mut self, headers: FieldTable) {
        self.message_prop.with_headers(headers);
    }

    // message ttl
    pub fn set_message_expiration(&mut self, seconds: usize) {
        let ms = (seconds * 1000).to_string();
        self.message_prop.with_expiration(&ms);
    }

    pub fn set_message_user_id(&mut self, user_id: &str) {
        self.message_prop.with_user_id(user_id);
    }

    pub fn set_message_app_id(&mut self, app_id: &str) {
        self.message_prop.with_app_id(app_id);
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
