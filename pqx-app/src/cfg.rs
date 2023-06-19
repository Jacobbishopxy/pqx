//! file: cfg.rs
//! author: Jacob Xie
//! date: 2023/06/18 00:47:30 Sunday
//! brief:

use std::collections::HashMap;

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
    use pqx::ec::util::{current_dir, join_dir};
    use pqx::pqx_util::read_yaml;

    use super::*;

    const INIT_CONFIG: &str = "init.template.yml";

    fn get_conn_yaml_path(filename: &str) -> std::path::PathBuf {
        join_dir(current_dir().unwrap(), filename).unwrap()
    }

    #[test]
    fn read_yaml_success() {
        let config_path = get_conn_yaml_path(INIT_CONFIG);
        let config_path = config_path.to_string_lossy();
        let config: InitiationsConfig = read_yaml(config_path).unwrap();

        println!("{:?}", config);
    }
}
