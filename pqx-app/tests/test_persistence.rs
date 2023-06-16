//! file: test_persistence.rs
//! author: Jacob Xie
//! date: 2023/06/16 15:54:59 Friday
//! brief:

use once_cell::sync::Lazy;
use pqx::ec::{util::*, CmdArg};
use pqx::mq::MqConn;
use pqx::pqx_util::db::{PersistClient, PersistConn};
use pqx::pqx_util::read_yaml;
use pqx_app::adt::{Command, ExecutionResult};
use pqx_app::persistence::MessagePersistent;
use serde::Deserialize;

// ================================================================================================
// helper
// ================================================================================================

// PANIC if file not found!
fn get_conn_yaml_path() -> std::path::PathBuf {
    join_dir(current_dir().unwrap(), "config.yml").unwrap()
}

// ================================================================================================
// static
// ================================================================================================

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    mq: MqConn,
    db: PersistConn,
}

static CONN: Lazy<PersistConn> = Lazy::new(|| {
    let pth = get_conn_yaml_path();

    let config: Config = read_yaml(pth.to_str().unwrap()).unwrap();

    config.db
});

// ================================================================================================
// Test
// ================================================================================================

#[tokio::test]
async fn connection_success() {
    let conn = CONN.clone();

    let mut db = PersistClient::new(conn);
    let res = db.connect().await;
    println!("{:?}", res);
    assert!(res.is_ok());
}

#[tokio::test]
async fn create_table_success() {
    let conn = CONN.clone();
    let mut db = PersistClient::new(conn);
    let _ = db.connect().await;

    let mp = MessagePersistent::new(db.db.unwrap());
    let res = mp.create_table().await;
    // if table already exists, then return error
    println!("{:?}", res);
}

#[tokio::test]
async fn insert_success() {
    let conn = CONN.clone();
    let mut db = PersistClient::new(conn);
    let _ = db.connect().await;

    let mp = MessagePersistent::new(db.db.unwrap());

    let cmd_arg = CmdArg::Ping {
        addr: "github.com".to_owned(),
    };
    let cmd = Command::new(cmd_arg);

    let res = mp.insert_history(&cmd).await;
    println!("{:?}", res);
    assert!(res.is_ok());
    let id = res.unwrap();

    let er = ExecutionResult::new(0);
    let res = mp.insert_result(id, &er).await;
    println!("{:?}", res);
    assert!(res.is_ok());
}

#[tokio::test]
async fn find_one_success() {
    let conn = CONN.clone();
    let mut db = PersistClient::new(conn);
    let _ = db.connect().await;

    let mp = MessagePersistent::new(db.db.unwrap());

    let res = mp.find_one(1).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    println!("command: {:?}", res.0);
    println!("execution_result: {:?}", res.1);
}
