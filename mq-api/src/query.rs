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
}
