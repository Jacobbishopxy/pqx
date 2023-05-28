//! file: publish.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:54:55 Friday
//! brief:

use amqprs::channel::BasicPublishArguments;
use amqprs::BasicProperties;
use serde::Serialize;

use crate::error::PqxResult;

use super::client::MqClient;

// ================================================================================================
// Publisher
// ================================================================================================

#[derive(Clone)]
pub struct Publisher<C>
where
    C: AsRef<MqClient>,
{
    client: C,
    message_prop: BasicProperties,
}

impl<C> Publisher<C>
where
    C: AsRef<MqClient>,
{
    pub fn new(client: C) -> Self {
        Self {
            client,
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
        self.client
            .as_ref()
            .channel()?
            .basic_publish(self.message_prop.clone(), content, args)
            .await?;

        Ok(())
    }
}
