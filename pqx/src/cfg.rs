//! file: cfg.rs
//! author: Jacob Xie
//! date: 2023/05/29 08:40:45 Monday
//! brief:

use std::fs::File;

use serde::{de::DeserializeOwned, Deserialize};

use crate::error::PqxResult;
use crate::mq::ConnArg;

// ================================================================================================
// read config file
// ================================================================================================

pub fn read_yaml<P, T>(path: P) -> PqxResult<T>
where
    P: AsRef<str>,
    T: DeserializeOwned,
{
    let f = File::open(path.as_ref())?;
    let d: T = serde_yaml::from_reader(f)?;

    Ok(d)
}

pub fn read_json<P, T>(path: P) -> PqxResult<T>
where
    P: AsRef<str>,
    T: DeserializeOwned,
{
    let f = File::open(path.as_ref())?;
    let d: T = serde_json::from_reader(f)?;

    Ok(d)
}

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
        let cfg: CfgMqConn = read_yaml(yaml_path.to_str().unwrap()).unwrap();

        println!("{:?}", cfg);
    }
}
