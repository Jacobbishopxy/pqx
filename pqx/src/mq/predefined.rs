//! file: predefined.rs
//! author: Jacob Xie
//! date: 2023/06/17 18:15:59 Saturday
//! brief:

use std::str::FromStr;

use amqprs::channel::{
    BasicAckArguments, BasicNackArguments, BasicPublishArguments, Channel, ExchangeType,
};
use amqprs::{BasicProperties, Deliver, FieldName, FieldTable, FieldValue};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::error::{PqxError, PqxResult};

// ================================================================================================
// const
// ================================================================================================

pub static X_DELAYED_MESSAGE: Lazy<ExchangeType> =
    Lazy::new(|| ExchangeType::Plugin(String::from("x-delayed-message")));

///////////////////////////////////////////////////////////////////////////////////////////////////

pub static X_DELAYED_TYPE: Lazy<FieldName> =
    Lazy::new(|| FieldName::try_from("x-delayed-type").unwrap());

pub static X_DELAY: Lazy<FieldName> = Lazy::new(|| FieldName::try_from("x-delay").unwrap());

pub static X_RETRIES: Lazy<FieldName> =
    Lazy::new(|| FieldName::try_from(String::from("x-retries")).unwrap());

pub static X_MATCH: Lazy<FieldName> = Lazy::new(|| FieldName::try_from("x-match").unwrap());

pub static X_MESSAGE_TTL: Lazy<FieldName> =
    Lazy::new(|| FieldName::try_from("x-message-ttl").unwrap());

pub static X_DEAD_LETTER_EXCHANGE: Lazy<FieldName> =
    Lazy::new(|| FieldName::try_from("x-dead-letter-exchange").unwrap());

pub static X_DEAD_ROUTING_KEY: Lazy<FieldName> =
    Lazy::new(|| FieldName::try_from("x-dead-routing-key").unwrap());

// ================================================================================================
// MatchType
// ================================================================================================

#[derive(Debug, Serialize, Deserialize)]
pub enum MatchType {
    Any,
    All,
}

impl ToString for MatchType {
    fn to_string(&self) -> String {
        match self {
            MatchType::Any => String::from("any"),
            MatchType::All => String::from("all"),
        }
    }
}

impl FromStr for MatchType {
    type Err = PqxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "any" => Ok(MatchType::Any),
            "all" => Ok(MatchType::All),
            _ => Err("match_type: any/all".into()),
        }
    }
}

// ================================================================================================
// field table insert
// ================================================================================================

pub struct FieldTableBuilder(FieldTable);

impl FieldTableBuilder {
    pub fn new() -> Self {
        Self(FieldTable::new())
    }

    pub fn finish(self) -> FieldTable {
        self.0
    }

    pub fn x_common_pair(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.0.insert(
            FieldName::try_from(key.into()).unwrap(),
            FieldValue::from(value.into()),
        );

        self
    }

    pub fn x_delayed_type(&mut self, exchange_type: &ExchangeType) -> &mut Self {
        self.0.insert(
            X_DELAYED_TYPE.clone(),
            FieldValue::from(exchange_type.to_string()),
        );

        self
    }

    pub fn x_delay(&mut self, delay: i32) -> &mut Self {
        self.0.insert(X_DELAY.clone(), FieldValue::I(delay));

        self
    }

    pub fn x_retries(&mut self, retries: i16) -> &mut Self {
        self.0.insert(X_RETRIES.clone(), FieldValue::s(retries));

        self
    }

    pub fn x_match(&mut self, t: &MatchType) -> &mut Self {
        self.0
            .insert(X_MATCH.clone(), FieldValue::from(t.to_string()));

        self
    }

    pub fn x_message_ttl(&mut self, ttl: i64) -> &mut Self {
        self.0.insert(X_MESSAGE_TTL.clone(), FieldValue::l(ttl));

        self
    }

    pub fn x_dead_letter_exchange(
        &mut self,
        exchange_name: impl Into<String>,
        routing_key: impl Into<String>,
    ) -> &mut Self {
        self.0.insert(
            X_DEAD_LETTER_EXCHANGE.clone(),
            FieldValue::from(exchange_name.into()),
        );
        self.0.insert(
            X_DEAD_ROUTING_KEY.clone(),
            FieldValue::from(routing_key.into()),
        );

        self
    }
}

// ================================================================================================
// field table getter
// ================================================================================================

pub struct FieldTableGetter<'a>(&'a FieldTable);

impl<'a> FieldTableGetter<'a> {
    pub fn new(ft: &'a FieldTable) -> Self {
        Self(ft)
    }

    pub fn x_common_pair(&self, key: impl Into<String>) -> PqxResult<String> {
        match self.0.get(&FieldName::try_from(key.into()).unwrap()) {
            Some(FieldValue::S(s)) => Ok(s.as_ref().clone()),
            None => Err("key doesn't exist".into()),
            _ => Err("key is not a string".into()),
        }
    }

    pub fn x_delay(&mut self) -> PqxResult<i32> {
        match self.0.get(&X_DELAY) {
            Some(FieldValue::I(d)) => Ok(*d),
            None => Err("x-delay doesn't exist".into()),
            _ => Err("x-delay is not a `i32`".into()),
        }
    }

    pub fn x_retries(&self) -> PqxResult<i16> {
        match self.0.get(&X_RETRIES) {
            Some(FieldValue::s(r)) => Ok(*r),
            None => Err("retries doesn't exist".into()),
            _ => Err("retries is not a `i16`".into()),
        }
    }

    pub fn x_match(&self) -> PqxResult<MatchType> {
        match self.0.get(&X_MATCH) {
            Some(FieldValue::S(s)) => Ok(MatchType::from_str(s.as_ref())?),
            None => Err("x-match doesn't exist".into()),
            _ => Err("x-match is not a string".into()),
        }
    }

    pub fn x_message_ttl(&self) -> PqxResult<i64> {
        match self.0.get(&X_MESSAGE_TTL) {
            Some(FieldValue::l(t)) => Ok(*t),
            None => Err("x-message-ttl doesn't exist".into()),
            _ => Err("x-message-ttl is not a `i64`".into()),
        }
    }

    pub fn x_dead_letter_exchange(&self) -> PqxResult<(String, String)> {
        let exchange_name = match self.0.get(&X_DEAD_LETTER_EXCHANGE) {
            Some(FieldValue::S(s)) => Ok(s.as_ref().clone()),
            None => Err("x-dead-letter-exchange doesn't exist"),
            _ => Err("x-dead-letter-exchange is not a string"),
        }?;

        let routing_key = match self.0.get(&X_DEAD_ROUTING_KEY) {
            Some(FieldValue::S(s)) => Ok(s.as_ref().clone()),
            None => Err("x-dead-routing-key doesn't exist"),
            _ => Err("x-dead-routing-key is not a string"),
        }?;

        Ok((exchange_name, routing_key))
    }
}

// ================================================================================================
// Retry
// ================================================================================================

pub struct Retry {
    pub exchange: String,
    pub routing_key: String,
    pub poke: u16,
    pub retries: u8,
}

impl Retry {
    pub async fn retry(
        &self,
        channel: &Channel,
        deliver: Deliver,
        mut props: BasicProperties,
        content: Vec<u8>,
    ) -> PqxResult<()> {
        // clone or create
        let mut headers = props.headers().cloned().unwrap_or(FieldTable::new());

        // if x_delay doesn't exist
        if let None = headers.get(&X_DELAY) {
            headers.insert(X_DELAY.clone(), FieldValue::I(self.poke.into()));
        }

        // if x_retries doesn't exist
        if let None = headers.get(&X_RETRIES) {
            headers.insert(X_RETRIES.clone(), FieldValue::s(self.retries.into()));
        }

        // consume 1 retry
        let retries = FieldTableGetter::new(&headers).x_retries().unwrap() - 1;

        if retries > 0 {
            // publish to delayed exchange and ack
            headers.remove(&X_RETRIES);
            headers.insert(X_RETRIES.clone(), FieldValue::s(retries));

            channel
                .basic_publish(
                    props.with_headers(headers).finish(),
                    content,
                    BasicPublishArguments::new(&self.exchange, &self.routing_key),
                )
                .await?;

            let _ = channel.basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false));
        } else {
            // discard message (if DLX is set, then goes to there)
            channel
                .basic_nack(BasicNackArguments::new(
                    deliver.delivery_tag(),
                    false,
                    false,
                ))
                .await?;
        }

        Ok(())
    }
}
