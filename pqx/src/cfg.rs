//! file: cfg.rs
//! author: Jacob Xie
//! date: 2023/05/29 08:40:45 Monday
//! brief:

use serde::Deserialize;

use crate::mq::ConnArg;

// ================================================================================================
// Configs
// ================================================================================================

#[derive(Debug, Deserialize)]
pub struct CfgMqConn {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub vhost: Option<String>,
}

impl<'a> From<&'a CfgMqConn> for ConnArg<'a> {
    fn from(v: &'a CfgMqConn) -> Self {
        ConnArg {
            host: v.host.as_ref(),
            port: v.port,
            user: v.user.as_ref(),
            pass: v.pass.as_ref(),
            vhost: v.vhost.as_deref(),
        }
    }
}

// ================================================================================================
// Test
// ================================================================================================

#[cfg(test)]
mod test_cfg {
    use super::*;
    use crate::ec::util::*;

    #[test]
    fn read_yaml_success() {
        let yaml_path = join_dir(current_dir().unwrap(), "conn.yml").unwrap();
        let cfg: CfgMqConn = pqx_util::read_yaml(yaml_path.to_str().unwrap()).unwrap();

        println!("{:?}", cfg);
    }
}
