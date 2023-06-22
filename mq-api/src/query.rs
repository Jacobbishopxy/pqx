//! file: query.rs
//! author: Jacob Xie
//! date: 2023/06/04 12:31:35 Sunday
//! brief:

use crate::MqClient;

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
// ================================================================================================

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
