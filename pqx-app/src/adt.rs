//! file: adt.rs
//! author: Jacob Xie
//! date: 2023/06/12 21:12:55 Monday
//! brief:

use anyhow::Error;
use pqx::amqprs::{FieldName, FieldTable, FieldValue};
use pqx::ec::CmdArg;
use serde::{Deserialize, Serialize};

// ================================================================================================
// Command
// ================================================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    consumer_ids: Vec<String>,
    retry: Option<usize>,
    poke: Option<usize>,
    timeout: Option<usize>,
    cmd: CmdArg,
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

        if let Some(t) = cmd.timeout {
            ft.insert(
                FieldName::try_from("x-message-ttl".to_string())?,
                FieldValue::l(i64::try_from(t)?),
            );
        }

        Ok(ft)
    }
}
