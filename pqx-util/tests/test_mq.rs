//! file: test_mq.rs
//! author: Jacob Xie
//! date: 2023/06/23 11:25:56 Friday
//! brief:

use once_cell::sync::Lazy;
use pqx_util::mq::*;
use pqx_util::read_yaml;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

// ================================================================================================
// Test MqClient
//
// development documentation (docker service of RMQ): `http://localhost:15672/api/index.html`
// ================================================================================================

#[tokio::test]
async fn simple_get() {
    let cfg: MqClientCfg = read_yaml("conn.yml").unwrap();

    let url = &cfg.url;
    let vhost = cfg.vhost.as_deref().unwrap_or("");
    let mut hm = HeaderMap::new();
    hm.insert(
        AUTHORIZATION,
        HeaderValue::from_str(cfg.auth.as_ref().unwrap()).unwrap(),
    );

    // client
    let client = MqClient::new_with_headers(url, vhost, hm);

    let res = client.get::<_, serde_json::Value>("overview").await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

// ================================================================================================
// Test MqQuery
// ================================================================================================

static CLIENT: Lazy<MqClient> =
    Lazy::new(|| MqClient::new_by_yaml("conn.yml").expect("config file exists"));

#[tokio::test]
async fn overview_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.overview().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn connections_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.connections().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn channels_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.channels().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn consumers_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.consumers().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn exchanges_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.exchanges().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn queues_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.queues().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn bindings_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.bindings().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn vhosts_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.vhosts().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn users_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.users().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn whoami_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.whoami().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn parameters_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.parameters().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn policies_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.policies().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn auth_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.auth().await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn exchanges_with_vhost_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.exchanges_with_vhost(&CLIENT.vhost()).await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn queues_with_vhost_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.queues_with_vhost(&CLIENT.vhost()).await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn bindings_with_vhost_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.bindings_with_vhost(&CLIENT.vhost()).await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}

#[tokio::test]
async fn policies_with_vhost_success() {
    let query = MqQuery::new(&CLIENT);

    let res = query.policies_with_vhost(&CLIENT.vhost()).await;
    assert!(res.is_ok());
    let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
    println!("{}", pretty_json);
}
