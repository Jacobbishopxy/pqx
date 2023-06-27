//! file: adt.rs
//! author: Jacob Xie
//! date: 2023/06/12 21:12:55 Monday
//! brief:

use std::collections::HashMap;

use chrono::Local;
use pqx::amqprs::BasicProperties;
use pqx::ec::CmdArg;
use pqx::error::PqxError;
use pqx::mq::FieldTableBuilder;
use pqx::pqx_custom_err;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entities::{message_history, message_result};

// ================================================================================================
// MailingTo & Command
// ================================================================================================

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub retry: Option<u8>,
    pub poke: Option<u16>,
    pub waiting_timeout: Option<u32>,
    pub consuming_timeout: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub mailing_to: Vec<HashMap<String, String>>,
    pub config: Config,
    pub cmd: CmdArg,
}

impl Command {
    pub fn new(cmd: CmdArg) -> Self {
        Self {
            mailing_to: Vec::new(),
            config: Config::default(),
            cmd,
        }
    }

    pub fn mailing_to(&self) -> &[HashMap<String, String>] {
        &self.mailing_to
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn cmd(&self) -> &CmdArg {
        &self.cmd
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

// Command -> ActiveModel
impl<'a> TryFrom<&'a Command> for message_history::ActiveModel {
    type Error = std::num::TryFromIntError;

    fn try_from(cmd: &'a Command) -> Result<Self, Self::Error> {
        let am = message_history::ActiveModel {
            mailing_to: Set(serde_json::json!(cmd.mailing_to)),
            retry: Set(cmd.config.retry.map(i16::from)),
            poke: Set(cmd.config.poke.map(i32::from)),
            waiting_timeout: Set(cmd.config.waiting_timeout.map(i64::from)),
            consuming_timeout: Set(cmd.config.consuming_timeout.map(i64::from)),
            cmd: Set(serde_json::json!(cmd.cmd)),
            time: Set(Local::now()),
            ..Default::default()
        };

        Ok(am)
    }
}

// Model -> Command
impl TryFrom<message_history::Model> for Command {
    type Error = PqxError;

    fn try_from(m: message_history::Model) -> Result<Self, Self::Error> {
        let res = Self {
            mailing_to: serde_json::from_value(m.mailing_to)?,
            config: Config {
                retry: m.retry.map(u8::try_from).transpose()?,
                poke: m.poke.map(u16::try_from).transpose()?,
                waiting_timeout: m.waiting_timeout.map(u32::try_from).transpose()?,
                consuming_timeout: m.consuming_timeout.map(u32::try_from).transpose()?,
            },
            cmd: serde_json::from_value(m.cmd)?,
        };

        Ok(res)
    }
}

// Command -> Vec<BasicProperties>
impl<'a> TryFrom<&'a Command> for Vec<BasicProperties> {
    type Error = PqxError;

    fn try_from(cmd: &'a Command) -> Result<Self, Self::Error> {
        let headers = {
            let mut ftb = FieldTableBuilder::new();

            if let Some(r) = cmd.config.retry {
                ftb.x_retries(r.into());
            }

            if let Some(p) = cmd.config.poke {
                // convert to milliseconds
                let p = i32::from(p) * 1000;
                ftb.x_delay(p);
            }

            if let Some(t) = cmd.config.waiting_timeout {
                // convert to milliseconds
                let t = i64::from(t) * 1000;
                ftb.x_message_ttl(t);
            }

            if let Some(t) = cmd.config.consuming_timeout {
                // convert to milliseconds
                let t = i64::from(t) * 1000;
                ftb.x_consume_ttl(t);
            }

            ftb.finish()
        };

        let mut res = vec![];

        for mt in cmd.mailing_to.iter() {
            let mut ftb = FieldTableBuilder::from(headers.clone());

            for (k, v) in mt.iter() {
                ftb.x_common_pair(k, v);
            }

            let mut props = BasicProperties::default();
            props.with_headers(ftb.finish());
            res.push(props);
        }

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

// ================================================================================================
// Test
// ================================================================================================

#[cfg(test)]
mod test_adt {
    use pqx::pqx_util::*;

    use super::*;

    const TASK: &str = "task.json";

    #[test]
    fn command_se_de_success() {
        // read task.json
        let task_path = get_cur_dir_file(TASK).unwrap();
        let task_path = task_path.to_string_lossy();
        let task = read_json::<_, Command>(task_path);

        println!("{:?}", task);
    }
}
