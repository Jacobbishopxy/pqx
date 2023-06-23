//! file: cfg.rs
//! author: Jacob Xie
//! date: 2023/06/18 00:47:30 Sunday
//! brief:

use std::collections::HashMap;

use pqx::mq::{MatchType, MqConn};
use pqx::pqx_util::{MqApiCfg, PersistConn};
use serde::Deserialize;

// ================================================================================================
// Connections config
// ================================================================================================

#[derive(Debug, Deserialize)]
pub struct ConnectionsConfig {
    pub mq: MqConn,
    pub db: PersistConn,
    pub mq_api: MqApiCfg,
}

// ================================================================================================
// Initiations config
// ================================================================================================

#[derive(Debug, Deserialize)]
pub struct HeaderQueue {
    pub queue: String,
    pub match_type: MatchType,
    pub kv: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct InitiationsConfig {
    pub header_exchange: String,
    pub header_queues: Vec<HeaderQueue>,
    pub delayed_exchange: String,
    pub dead_letter_exchange: String,
}

// ================================================================================================
// Test
// ================================================================================================

#[cfg(test)]
mod test_cfg {
    use pqx::pqx_util::{get_cur_dir_file, read_yaml};

    use super::*;

    const INIT_CONFIG: &str = "init.template.yml";

    #[test]
    fn read_yaml_success() {
        let config_path = get_cur_dir_file(INIT_CONFIG).unwrap();
        let config_path = config_path.to_string_lossy();
        let config: InitiationsConfig = read_yaml(config_path).unwrap();

        println!("{:?}", config);
    }
}
