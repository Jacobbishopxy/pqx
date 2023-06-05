//! file: client.rs
//! author: Jacob Xie
//! date: 2023/06/04 08:51:50 Sunday
//! brief:

use pqx_util::{read_json, read_yaml};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    ClientBuilder, Url,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::MqApiResult;

// ================================================================================================
// MqClientCfg
// ================================================================================================

#[derive(Debug, Deserialize)]
pub struct MqClientCfg {
    host: String,
    port: usize,
    path: Option<String>,
    auth: Option<String>,
}

// ================================================================================================
// MqClient
// ================================================================================================

#[derive(Debug)]
pub struct MqClient {
    url: String,
    client: reqwest::Client,
}

impl MqClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            client: ClientBuilder::new().build().expect("build success"),
        }
    }

    pub fn new_with_headers(url: &str, headers: HeaderMap) -> Self {
        let c = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("build success");

        Self {
            url: url.to_owned(),
            client: c,
        }
    }

    pub fn new_by_yaml(path: impl AsRef<str>) -> MqApiResult<Self> {
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
        let url = format!("{}:{}", cfg.host, cfg.port);
        let url = match cfg.path {
            Some(p) => format!("{}/{}", url, p),
            None => url,
        };

        Ok(Self { url, client: c })
    }

    pub fn new_by_json(path: impl AsRef<str>) -> MqApiResult<Self> {
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
        let url = format!("{}:{}", cfg.host, cfg.port);
        let url = match cfg.path {
            Some(p) => format!("{}/{}", url, p),
            None => url,
        };

        Ok(Self { url, client: c })
    }

    pub async fn get<P: AsRef<str>, R: DeserializeOwned>(&self, path: P) -> MqApiResult<R> {
        let pth = format!("{}/{}", self.url, path.as_ref());
        let encoded = parse_url(pth)?;
        let res = self.client.get(encoded).send().await?.json::<R>().await?;

        Ok(res)
    }

    pub async fn post<P: AsRef<str>, T: Serialize, R: DeserializeOwned>(
        &self,
        path: P,
        req: &T,
    ) -> MqApiResult<R> {
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
    ) -> MqApiResult<R> {
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
// helper
// ================================================================================================

fn parse_url<T: AsRef<str>>(url: T) -> MqApiResult<Url> {
    Ok(Url::parse(url.as_ref()).map_err(|_| "url encoded fail")?)
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

        let client = MqClient::new_with_headers("http://localhost:15672/api", hm);

        let res = client.get::<_, serde_json::Value>("overview").await;
        assert!(res.is_ok());
        let pretty_json = serde_json::to_string_pretty(&res.unwrap()).unwrap();
        println!("{}", pretty_json);
    }
}
