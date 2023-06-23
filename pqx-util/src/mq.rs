//! file: mq.rs
//! author: Jacob Xie
//! date: 2023/06/23 10:46:47 Friday
//! brief:

use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    ClientBuilder, Url,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{read_json, read_yaml, PqxUtilResult};

// ================================================================================================
// helper
// ================================================================================================

fn parse_url<T: AsRef<str>>(url: T) -> PqxUtilResult<Url> {
    Ok(Url::parse(url.as_ref()).map_err(|_| "url encoded fail")?)
}

// ================================================================================================
// MqClientCfg
// ================================================================================================

#[derive(Debug, Deserialize)]
pub struct MqClientCfg {
    url: String,
    auth: Option<String>,
    vhost: Option<String>,
}

// ================================================================================================
// MqClient
// ================================================================================================

#[derive(Debug)]
pub struct MqClient {
    url: String,
    vhost: String,
    client: reqwest::Client,
}

impl MqClient {
    pub fn new(url: &str, vhost: &str) -> Self {
        Self {
            url: url.to_owned(),
            vhost: vhost.to_owned(),
            client: ClientBuilder::new().build().expect("build success"),
        }
    }

    pub fn new_with_headers(url: &str, vhost: &str, headers: HeaderMap) -> Self {
        let c = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("build success");

        Self {
            url: url.to_owned(),
            vhost: vhost.to_owned(),
            client: c,
        }
    }

    pub fn new_by_yaml(path: impl AsRef<str>) -> PqxUtilResult<Self> {
        let cfg: MqClientCfg = read_yaml(path)?;

        let c = match cfg.auth {
            Some(ref auth) => {
                let cb = ClientBuilder::new();
                let mut hm = HeaderMap::new();
                let v = HeaderValue::from_str(auth).map_err(|_| "invalid header value")?;
                hm.insert(AUTHORIZATION, v);

                cb.default_headers(hm)
            }
            None => ClientBuilder::default(),
        }
        .build()?;

        Ok(Self {
            url: cfg.url,
            vhost: cfg.vhost.unwrap_or(String::from("")),
            client: c,
        })
    }

    pub fn new_by_json(path: impl AsRef<str>) -> PqxUtilResult<Self> {
        let cfg: MqClientCfg = read_json(path)?;

        let c = match cfg.auth {
            Some(ref auth) => {
                let cb = ClientBuilder::new();
                let mut hm = HeaderMap::new();
                let v = HeaderValue::from_str(auth).map_err(|_| "invalid header value")?;
                hm.insert(AUTHORIZATION, v);

                cb.default_headers(hm)
            }
            None => ClientBuilder::default(),
        }
        .build()?;

        Ok(Self {
            url: cfg.url,
            vhost: cfg.vhost.unwrap_or(String::from("")),
            client: c,
        })
    }

    pub fn vhost(&self) -> &str {
        self.vhost.as_ref()
    }

    pub async fn get<P: AsRef<str>, R: DeserializeOwned>(&self, path: P) -> PqxUtilResult<R> {
        let pth = format!("{}/{}", self.url, path.as_ref());
        let encoded = parse_url(pth)?;
        let res = self.client.get(encoded).send().await?.json::<R>().await?;

        Ok(res)
    }

    pub async fn post<P: AsRef<str>, T: Serialize, R: DeserializeOwned>(
        &self,
        path: P,
        req: &T,
    ) -> PqxUtilResult<R> {
        let pth = format!("{}/{}", self.url, path.as_ref());
        let encoded = parse_url(pth)?;
        let res = self
            .client
            .post(encoded)
            .json(req)
            .send()
            .await?
            .json::<R>()
            .await?;

        Ok(res)
    }

    pub async fn put<P: AsRef<str>, T: Serialize, R: DeserializeOwned>(
        &self,
        path: P,
        req: &T,
    ) -> PqxUtilResult<R> {
        let pth = format!("{}/{}", self.url, path.as_ref());
        let encoded = parse_url(pth)?;
        let res = self
            .client
            .put(encoded)
            .json(req)
            .send()
            .await?
            .json::<R>()
            .await?;

        Ok(res)
    }
}

// ================================================================================================
// MqQuery
// ================================================================================================

#[derive(Debug)]
pub struct MqQuery<'a> {
    client: &'a MqClient,
}

impl<'a> MqQuery<'a> {
    pub fn new(client: &'a MqClient) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &MqClient {
        self.client
    }

    // ============================================================================================
    // biz
    // ============================================================================================

    impl_simple_get!(overview, "overview");
    impl_simple_get!(connections, "connections");
    impl_simple_get!(channels, "channels");
    impl_simple_get!(consumers, "consumers");
    impl_simple_get!(exchanges, "exchanges");
    impl_simple_get!(queues, "queues");
    impl_simple_get!(bindings, "bindings");
    impl_simple_get!(vhosts, "vhosts");
    impl_simple_get!(users, "users");
    impl_simple_get!(whoami, "whoami");
    impl_simple_get!(parameters, "parameters");
    impl_simple_get!(policies, "policies");
    impl_simple_get!(auth, "auth");

    impl_get_with_vhost!(exchanges, "exchanges");
    impl_get_with_vhost!(queues, "queues");
    impl_get_with_vhost!(bindings, "bindings");
    impl_get_with_vhost!(policies, "policies");
}

// ================================================================================================
// test
//
// development documentation (docker service of RMQ): `http://localhost:15672/api/index.html`
// ================================================================================================

#[cfg(test)]
mod test_client {
    use once_cell::sync::Lazy;
    use reqwest::header::{HeaderValue, AUTHORIZATION};

    use super::*;

    static HEADER: Lazy<HeaderMap> = Lazy::new(|| {
        let mut hm = HeaderMap::new();
        hm.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Basic YWRtaW46YWRtaW4="), // admin:admin
        );

        hm
    });

    #[tokio::test]
    async fn simple_get() {
        let hm = HEADER.clone();

        let client = MqClient::new_with_headers("http://localhost:15672/api", "", hm);

        let res = client.get::<_, serde_json::Value>("overview").await;
        assert!(res.is_ok());
        let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
        println!("{}", pretty_json);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test_query {
    use once_cell::sync::Lazy;

    use super::*;

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
}
