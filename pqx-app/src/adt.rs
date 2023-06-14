//! file: adt.rs
//! author: Jacob Xie
//! date: 2023/06/12 21:12:55 Monday
//! brief:

use anyhow::Error;
use pqx::amqprs::{FieldName, FieldTable, FieldValue};
use pqx::ec::CmdArg;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::entities::message_history;

// ================================================================================================
// Command
// ================================================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub consumer_ids: Vec<String>,
    pub retry: Option<usize>,
    pub poke: Option<usize>,
    pub waiting_timeout: Option<usize>,
    pub consuming_timeout: Option<usize>,
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
    type Error = Error;

    fn try_from(cmd: &'a Command) -> Result<Self, Self::Error> {
        let mut ft = FieldTable::new();

        if let Some(r) = cmd.retry {
            ft.insert(
                FieldName::try_from("x-retry".to_string())?,
                FieldValue::s(i16::try_from(r)?),
            );
        }

        if let Some(p) = cmd.poke {
            ft.insert(
                FieldName::try_from("x-delay".to_string())?,
                FieldValue::I(i32::try_from(p)?),
            );
        }

        if let Some(t) = cmd.waiting_timeout {
            ft.insert(
                FieldName::try_from("x-message-ttl".to_string())?,
                FieldValue::l(i64::try_from(t)?),
            );
        }

        if let Some(t) = cmd.consuming_timeout {
            ft.insert(
                FieldName::try_from("x-consume-ttl".to_string())?,
                FieldValue::l(i64::try_from(t)?),
            );
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
            retry: Set(cmd.retry.map(i16::try_from).transpose()?),
            poke: Set(cmd.poke.map(i32::try_from).transpose()?),
            waiting_timeout: Set(cmd.waiting_timeout.map(i64::try_from).transpose()?),
            consuming_timeout: Set(cmd.consuming_timeout.map(i64::try_from).transpose()?),
            cmd: Set(serde_json::json!(cmd.cmd)),
            ..Default::default()
        };

        Ok(am)
    }
}
