//! file: client.rs
//! author: Jacob Xie
//! date: 2023/06/04 08:51:50 Sunday
//! brief:

use reqwest::{ClientBuilder, Url};
use serde::{de::DeserializeOwned, Serialize};

use crate::MqApiResult;

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

    pub async fn _get<P: AsRef<str>, R: DeserializeOwned>(&self, path: P) -> MqApiResult<R> {
        let pth = format!("{}/{}", self.url, path.as_ref());
        let encoded = parse_url(pth)?;
        let res = self.client.get(encoded).send().await?.json::<R>().await?;

        Ok(res)
    }

    pub async fn _post<P: AsRef<str>, T: Serialize, R: DeserializeOwned>(
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

    pub async fn _put<P: AsRef<str>, T: Serialize, R: DeserializeOwned>(
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
