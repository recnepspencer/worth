//! Fault injection under the authorization fixture's open-owner assumption.

use super::authorization_world_installation::AuthorizationWorld;

impl AuthorizationWorld {
    pub(in crate::domain_computation::primary_graph) fn fail_next_durable_append(&self) {
        self.application
            .fail_next_durable_append_for_test()
            .expect("the authorization fixture keeps its owner open");
    }
}
