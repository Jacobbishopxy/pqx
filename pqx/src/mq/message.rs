//! file: message.rs
//! author: Jacob Xie
//! date: 2023/05/27 20:57:27 Saturday
//! brief:

use serde::{Deserialize, Serialize};

use crate::ec::cmd::CmdArg;
use crate::error::PqxError;

// ================================================================================================
// Mq messages
// ================================================================================================

#[derive(Debug, Serialize, Deserialize)]
pub enum PqxMessage {
    Undefined {
        msg: String,
    },
    Ping {
        addr: String,
    },
    Bash {
        cmd: String,
    },
    Ssh {
        ip: String,
        cmd: String,
        user: Option<String>,
    },
    CondaPython {
        env: String,
        dir: String,
        script: String,
    },
}

impl<'a> TryFrom<&'a PqxMessage> for CmdArg<'a> {
    type Error = PqxError;

    fn try_from(value: &'a PqxMessage) -> Result<Self, Self::Error> {
        match value {
            PqxMessage::Ping { addr } => Ok(CmdArg::Ping { addr }),
            PqxMessage::Bash { cmd } => Ok(CmdArg::Bash { cmd }),
            PqxMessage::Ssh { ip, cmd, user } => Ok(CmdArg::Ssh {
                ip,
                cmd,
                user: user.as_deref(),
            }),
            PqxMessage::CondaPython { env, dir, script } => {
                Ok(CmdArg::CondaPython { env, dir, script })
            }
            _ => Err(PqxError::Custom("undefined message")),
        }
    }
}

impl TryFrom<Vec<u8>> for PqxMessage {
    type Error = PqxError;

    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        let arg: PqxMessage = serde_json::from_slice(v.as_ref())?;

        Ok(arg)
    }
}

impl TryFrom<PqxMessage> for Vec<u8> {
    type Error = PqxError;

    fn try_from(v: PqxMessage) -> Result<Self, Self::Error> {
        let arg: Vec<u8> = serde_json::to_vec(&v)?;

        Ok(arg)
    }
}
