//! file: adt.rs
//! author: Jacob Xie
//! date: 2023/06/12 21:12:55 Monday
//! brief:

use std::collections::HashMap;

use chrono::Local;
use pqx::amqprs::{FieldTable, FieldValue};
use pqx::ec::CmdArg;
use pqx::error::PqxError;
use pqx::mq::{X_CONSUME_TTL, X_DELAY, X_MESSAGE_TTL, X_RETRIES};
use pqx::pqx_custom_err;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entities::{message_history, message_result};

// ================================================================================================
// MailingTo & Command
// ================================================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MailingTo {
    Any(HashMap<String, String>),
    All(HashMap<String, String>),
}

impl Default for MailingTo {
    fn default() -> Self {
        Self::Any(HashMap::new())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub mailing_to: MailingTo,
    pub retry: Option<u8>,
    pub poke: Option<u16>,
    pub waiting_timeout: Option<u32>,
    pub consuming_timeout: Option<u32>,
    pub cmd: CmdArg,
}

impl Command {
    pub fn new(cmd: CmdArg) -> Self {
        Self {
            mailing_to: MailingTo::default(),
            retry: None,
            poke: None,
            waiting_timeout: None,
            consuming_timeout: None,
            cmd,
        }
    }

    pub fn cmd(&self) -> &CmdArg {
        &self.cmd
    }
}

impl<'a> TryFrom<&'a Command> for FieldTable {
    type Error = PqxError;

    fn try_from(cmd: &'a Command) -> Result<Self, Self::Error> {
        let mut ft = FieldTable::new();

        if let Some(r) = cmd.retry {
            ft.insert(X_RETRIES.clone(), FieldValue::s(r.into()));
        }

        if let Some(p) = cmd.poke {
            // convert to milliseconds
            let p = i32::from(p) * 1000;
            ft.insert(X_DELAY.clone(), FieldValue::I(p));
        }

        if let Some(t) = cmd.waiting_timeout {
            // convert to milliseconds
            let t = i64::from(t) * 1000;
            ft.insert(X_MESSAGE_TTL.clone(), FieldValue::l(t));
        }

        if let Some(t) = cmd.consuming_timeout {
            // convert to milliseconds
            let t = i64::from(t) * 1000;
            ft.insert(X_CONSUME_TTL.clone(), FieldValue::l(t));
        }

        Ok(ft)
    }
}

// Command -> ActiveModel
impl<'a> TryFrom<&'a Command> for message_history::ActiveModel {
    type Error = std::num::TryFromIntError;

    fn try_from(cmd: &'a Command) -> Result<Self, Self::Error> {
        let am = message_history::ActiveModel {
            mailing_to: Set(serde_json::json!(cmd.mailing_to)),
            retry: Set(cmd.retry.map(i16::from)),
            poke: Set(cmd.poke.map(i32::from)),
            waiting_timeout: Set(cmd.waiting_timeout.map(i64::from)),
            consuming_timeout: Set(cmd.consuming_timeout.map(i64::from)),
            cmd: Set(serde_json::json!(cmd.cmd)),
            time: Set(Local::now()),
            ..Default::default()
        };

        Ok(am)
    }
}

impl TryFrom<message_history::Model> for Command {
    type Error = PqxError;

    fn try_from(m: message_history::Model) -> Result<Self, Self::Error> {
        let res = Self {
            mailing_to: serde_json::from_value(m.mailing_to)?,
            retry: m.retry.map(u8::try_from).transpose()?,
            poke: m.poke.map(u16::try_from).transpose()?,
            waiting_timeout: m.waiting_timeout.map(u32::try_from).transpose()?,
            consuming_timeout: m.consuming_timeout.map(u32::try_from).transpose()?,
            cmd: serde_json::from_value(m.cmd)?,
        };

        Ok(res)
    }
}

// ================================================================================================
// ExecutionResult
// ================================================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub result: Option<String>,
}

impl ExecutionResult {
    pub fn new(exit_code: i32) -> Self {
        Self {
            exit_code,
            result: None,
        }
    }

    pub fn new_with_result(exit_code: i32, result: impl Into<String>) -> Self {
        Self {
            exit_code,
            result: Some(result.into()),
        }
    }

    pub fn into_active_model(&self, history_id: i64) -> message_result::ActiveModel {
        message_result::ActiveModel {
            history_id: Set(history_id),
            exit_code: Set(self.exit_code),
            result: Set(self.result.clone()),
            time: Set(Local::now()),
            ..Default::default()
        }
    }
}

impl From<message_result::Model> for ExecutionResult {
    fn from(m: message_result::Model) -> Self {
        Self {
            exit_code: m.exit_code,
            result: m.result,
        }
    }
}

// ================================================================================================
// Inspection result
// ================================================================================================

#[derive(Debug)]
pub struct ExchangeInfo {
    pub name: String,
    pub exchange_type: String,
    pub auto_delete: bool,
    pub durable: bool,
    pub internal: bool,
    pub vhost: String,
    pub arguments: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct QueueInfo {
    pub name: String,
    pub queue_type: String,
    pub state: String,
    pub vhost: String,
    pub consumers: i64,
    pub consumer_capacity: i64,
    pub consumer_utilisation: i64,
    pub exclusive: bool,
    pub durable: bool,
    pub arguments: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct BindingInfo {
    pub source: String,
    pub destination: String,
    pub destination_type: String,
    pub properties_key: String,
    pub routing_key: String,
    pub arguments: HashMap<String, Value>,
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'a> TryFrom<&'a Value> for ExchangeInfo {
    type Error = PqxError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        let object = value.as_object().ok_or(pqx_custom_err!("ExchangeInfo"))?;

        let name = object
            .get("name")
            .ok_or(pqx_custom_err!("name"))?
            .as_str()
            .ok_or(pqx_custom_err!("str"))?
            .to_owned();
        let exchange_type = object
            .get("type")
            .ok_or(pqx_custom_err!("type"))?
            .as_str()
            .ok_or(pqx_custom_err!("str"))?
            .to_owned();
        let auto_delete = object
            .get("auto_delete")
            .ok_or(pqx_custom_err!("auto_delete"))?
            .as_bool()
            .ok_or(pqx_custom_err!("bool"))?;
        let durable = object
            .get("durable")
            .ok_or(pqx_custom_err!("durable"))?
            .as_bool()
            .ok_or(pqx_custom_err!("bool"))?;
        let internal = object
            .get("internal")
            .ok_or(pqx_custom_err!("internal"))?
            .as_bool()
            .ok_or(pqx_custom_err!("bool"))?;
        let vhost = object
            .get("vhost")
            .ok_or(pqx_custom_err!("vhost"))?
            .as_str()
            .ok_or(pqx_custom_err!("str"))?
            .to_owned();
        let arguments: HashMap<String, Value> = serde_json::from_value(
            object
                .get("arguments")
                .ok_or(pqx_custom_err!("arguments"))?
                .clone(),
        )?;

        Ok(Self {
            name,
            exchange_type,
            auto_delete,
            durable,
            internal,
            vhost,
            arguments,
        })
    }
}

impl<'a> TryFrom<&'a Value> for QueueInfo {
    type Error = PqxError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        let object = value.as_object().ok_or(pqx_custom_err!("QueueInfo"))?;

        let name = object
            .get("name")
            .ok_or(pqx_custom_err!("name"))?
            .as_str()
            .ok_or(pqx_custom_err!("str"))?
            .to_owned();
        let queue_type = object
            .get("type")
            .ok_or(pqx_custom_err!("type"))?
            .as_str()
            .ok_or(pqx_custom_err!("str"))?
            .to_owned();
        let state = object
            .get("state")
            .ok_or(pqx_custom_err!("state"))?
            .as_str()
            .ok_or(pqx_custom_err!("str"))?
            .to_owned();
        let vhost = object
            .get("vhost")
            .ok_or(pqx_custom_err!("vhost"))?
            .as_str()
            .ok_or(pqx_custom_err!("str"))?
            .to_owned();
        let consumers = object
            .get("consumers")
            .ok_or(pqx_custom_err!("consumers"))?
            .as_i64()
            .ok_or(pqx_custom_err!("i64"))?;
        let consumer_capacity = object
            .get("consumer_capacity")
            .ok_or(pqx_custom_err!("consumer_capacity"))?
            .as_i64()
            .ok_or(pqx_custom_err!("i64"))?;
        let consumer_utilisation = object
            .get("consumer_utilisation")
            .ok_or(pqx_custom_err!("consumer_utilisation"))?
            .as_i64()
            .ok_or(pqx_custom_err!("i64"))?;
        let exclusive = object
            .get("exclusive")
            .ok_or(pqx_custom_err!("exclusive"))?
            .as_bool()
            .ok_or(pqx_custom_err!("bool"))?
            .to_owned();
        let durable = object
            .get("durable")
            .ok_or(pqx_custom_err!("durable"))?
            .as_bool()
            .ok_or(pqx_custom_err!("bool"))?
            .to_owned();
        let arguments: HashMap<String, Value> = serde_json::from_value(
            object
                .get("arguments")
                .ok_or(pqx_custom_err!("arguments"))?
                .clone(),
        )?;

        Ok(Self {
            name,
            queue_type,
            state,
            vhost,
            consumers,
            consumer_capacity,
            consumer_utilisation,
            exclusive,
            durable,
            arguments,
        })
    }
}

impl<'a> TryFrom<&'a Value> for BindingInfo {
    type Error = PqxError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        let object = value.as_object().ok_or(pqx_custom_err!("QueueInfo"))?;
        let source = object
            .get("source")
            .ok_or(pqx_custom_err!("source"))?
            .as_str()
            .ok_or(pqx_custom_err!("source"))?
            .to_owned();
        let destination = object
            .get("destination")
            .ok_or(pqx_custom_err!("destination"))?
            .as_str()
            .ok_or(pqx_custom_err!("destination"))?
            .to_owned();
        let destination_type = object
            .get("destination_type")
            .ok_or(pqx_custom_err!("destination_type"))?
            .as_str()
            .ok_or(pqx_custom_err!("destination_type"))?
            .to_owned();
        let properties_key = object
            .get("properties_key")
            .ok_or(pqx_custom_err!("properties_key"))?
            .as_str()
            .ok_or(pqx_custom_err!("properties_key"))?
            .to_owned();
        let routing_key = object
            .get("routing_key")
            .ok_or(pqx_custom_err!("routing_key"))?
            .as_str()
            .ok_or(pqx_custom_err!("routing_key"))?
            .to_owned();
        let arguments: HashMap<String, Value> = serde_json::from_value(
            object
                .get("arguments")
                .ok_or(pqx_custom_err!("arguments"))?
                .clone(),
        )?;

        Ok(Self {
            source,
            destination,
            destination_type,
            properties_key,
            routing_key,
            arguments,
        })
    }
}
