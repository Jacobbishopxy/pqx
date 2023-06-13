//! file: persistence.rs
//! author: Jacob Xie
//! date: 2023/06/13 11:02:30 Tuesday
//! brief:

use std::time::Duration;

use anyhow::{anyhow, Result};
use pqx::pqx_util::{read_json, read_yaml};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};

// ================================================================================================
// PersistConn
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistConn {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub db: String,
}

impl From<PersistConn> for ConnectOptions {
    fn from(conn: PersistConn) -> Self {
        ConnectOptions::new(format!(
            "{}://{}:{}@{}:{}/{}",
            conn.protocol, conn.user, conn.pass, conn.host, conn.port, conn.db
        ))
    }
}

// ================================================================================================
// PersistClient
// ================================================================================================

pub struct PersistClient {
    conn: ConnectOptions,
    db: Option<DatabaseConnection>,
}

impl PersistClient {
    pub fn new(conn: PersistConn) -> Self {
        Self {
            conn: conn.into(),
            db: None,
        }
    }

    pub fn new_by_json(path: impl AsRef<str>) -> Result<Self> {
        let conn: PersistConn = read_json(path)?;

        Ok(Self::new(conn))
    }

    pub fn new_by_yaml(path: impl AsRef<str>) -> Result<Self> {
        let conn: PersistConn = read_yaml(path)?;

        Ok(Self::new(conn))
    }

    pub fn conn(&self) -> &ConnectOptions {
        &self.conn
    }

    pub fn url(&self) -> &str {
        self.conn.get_url()
    }

    pub fn with_max_connection(&mut self, size: u32) -> &mut Self {
        self.conn.max_connections(size);

        self
    }

    pub fn with_min_connection(&mut self, size: u32) -> &mut Self {
        self.conn.min_connections(size);

        self
    }

    pub fn with_connect_timeout(&mut self, secs: u64) -> &mut Self {
        self.conn.connect_timeout(Duration::from_secs(secs));

        self
    }

    pub fn with_acquire_timeout(&mut self, secs: u64) -> &mut Self {
        self.conn.acquire_timeout(Duration::from_secs(secs));

        self
    }

    pub fn with_max_lifetime(&mut self, secs: u64) -> &mut Self {
        self.conn.max_lifetime(Duration::from_secs(secs));

        self
    }

    pub async fn connect(&mut self) -> Result<()> {
        if self.db.is_some() {
            return Err(anyhow!("connection already exists"));
        }

        self.db = Some(Database::connect(self.conn.clone()).await?);

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if self.db.is_none() {
            return Err(anyhow!("connection has not established"));
        }

        self.db.take().unwrap().close().await?;

        Ok(())
    }
}
