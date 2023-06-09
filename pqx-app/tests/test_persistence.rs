//! file: test_persistence.rs
//! author: Jacob Xie
//! date: 2023/06/16 15:54:59 Friday
//! brief:

use once_cell::sync::Lazy;
use pqx::ec::CmdArg;
use pqx::pqx_util::{get_cur_dir_file, read_yaml, PersistClient, PersistConn};
use pqx_app::adt::{Command, ExecutionResult};
use pqx_app::cfg::ConnectionsConfig;
use pqx_app::persist::MessagePersistent;

// ================================================================================================
// static
// ================================================================================================

static CONN: Lazy<PersistConn> = Lazy::new(|| {
    let pth = get_cur_dir_file("conn.yml").unwrap();

    let config: ConnectionsConfig = read_yaml(pth.to_str().unwrap()).unwrap();

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
async fn check_exists_success() {
    let conn = CONN.clone();
    let mut db = PersistClient::new(conn);
    let _ = db.connect().await;

    let mp = MessagePersistent::new(db.db.unwrap());

    let res = mp.check_existence().await;
    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
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

#[tokio::test]
async fn find_many_success() {
    let conn = CONN.clone();
    let mut db = PersistClient::new(conn);
    let _ = db.connect().await;

    let mp = MessagePersistent::new(db.db.unwrap());

    let res = mp.find_many([1, 3]).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    println!("res: {:?}", res);
}

#[tokio::test]
async fn find_by_pagination_success() {
    let conn = CONN.clone();
    let mut db = PersistClient::new(conn);
    let _ = db.connect().await;

    let mp = MessagePersistent::new(db.db.unwrap());

    let res = mp.find_by_pagination(0, 10).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    println!("res: {:?}", res);
}

#[tokio::test]
async fn drop_tables_success() {
    let conn = CONN.clone();
    let mut db = PersistClient::new(conn);
    let _ = db.connect().await;

    let mp = MessagePersistent::new(db.db.unwrap());

    let res = mp.drop_table().await;
    assert!(res.is_ok());
    println!("res: {:?}", res);
}
