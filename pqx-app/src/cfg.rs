//! file: cfg.rs
//! author: Jacob Xie
//! date: 2023/06/18 00:47:30 Sunday
//! brief:

use pqx::mq::{MatchType, MqConn};
use pqx::pqx_util::PersistConn;
use serde::Deserialize;

// ================================================================================================
// Connections config
// ================================================================================================

#[derive(Debug, Deserialize)]
pub struct ConnectionsConfig {
    pub mq: MqConn,
    pub db: PersistConn,
}

// ================================================================================================
// Initiations config
// ================================================================================================

#[derive(Debug, Deserialize)]
pub struct KV {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct HeaderQueue {
    pub queue: String,
    pub match_type: MatchType,
    pub kv: Vec<KV>,
}

#[derive(Debug, Deserialize)]
pub struct InitiationsConfig {
    pub exchange: String,
    pub header_queues: Vec<HeaderQueue>,
    pub delayed_exchange: String,
    pub dead_letter_exchange: String,
}
