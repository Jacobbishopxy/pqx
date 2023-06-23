//! file: adt.rs
//! author: Jacob Xie
//! date: 2023/06/12 21:12:55 Monday
//! brief:

use chrono::Local;
use pqx::amqprs::{FieldTable, FieldValue};
use pqx::ec::CmdArg;
use pqx::error::PqxError;
use pqx::mq::{X_CONSUME_TTL, X_DELAY, X_MESSAGE_TTL, X_RETRIES};
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::entities::{message_history, message_result};

// ================================================================================================
// Command
// ================================================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub consumer_ids: Vec<String>,
    pub retry: Option<u8>,
    pub poke: Option<u16>,
    pub waiting_timeout: Option<u32>,
    pub consuming_timeout: Option<u32>,
    pub cmd: CmdArg,
}

impl Command {
    pub fn new(cmd: CmdArg) -> Self {
        Self {
            consumer_ids: vec![],
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
            consumer_ids: Set(cmd.consumer_ids.join(",")),
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
            consumer_ids: m
                .consumer_ids
                .split(",")
                .map(String::from)
                .collect::<Vec<_>>(),
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
