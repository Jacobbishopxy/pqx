//! file: predefined.rs
//! author: Jacob Xie
//! date: 2023/06/17 18:15:59 Saturday
//! brief:

use amqprs::channel::ExchangeType;
use amqprs::{FieldName, FieldTable, FieldValue};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// ================================================================================================
// adt
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

pub static DELAYED_EXCHANGE: Lazy<ExchangeType> =
    Lazy::new(|| ExchangeType::Plugin(String::from("x-delayed-message")));

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

    pub fn x_delayed_type(&mut self) -> &mut Self {
        self.0.insert(
            FieldName::try_from("x-delayed-type").unwrap(),
            FieldValue::from(String::from("direct")),
        );

        self
    }

    pub fn x_delay(&mut self, delay: i32) -> &mut Self {
        self.0.insert(
            FieldName::try_from("x-delay").unwrap(),
            FieldValue::I(delay),
        );

        self
    }

    pub fn x_retries(&mut self, retries: i16) -> &mut Self {
        self.0.insert(
            FieldName::try_from(String::from("x-retries")).unwrap(),
            FieldValue::s(retries),
        );

        self
    }

    pub fn x_match(&mut self, t: &MatchType) -> &mut Self {
        self.0.insert(
            FieldName::try_from("x-match").unwrap(),
            FieldValue::from(t.to_string()),
        );

        self
    }

    pub fn x_message_ttl(&mut self, ttl: i64) -> &mut Self {
        self.0.insert(
            FieldName::try_from("x-message-ttl").unwrap(),
            FieldValue::l(ttl),
        );

        self
    }

    pub fn x_dead_letter_exchange(
        &mut self,
        exchange_name: impl Into<String>,
        routing_key: impl Into<String>,
    ) -> &mut Self {
        self.0.insert(
            FieldName::try_from("x-dead-letter-exchange").unwrap(),
            FieldValue::from(exchange_name.into()),
        );
        self.0.insert(
            FieldName::try_from("x-dead-routing-key").unwrap(),
            FieldValue::from(routing_key.into()),
        );

        self
    }
}
