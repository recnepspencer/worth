//! Open-owner assumption for authorization-world cleanup inspection.

use super::authorization_world_installation::AuthorizationWorld;

impl AuthorizationWorld {
    pub(in crate::domain_computation::primary_graph) fn open_pending_cleanup(
        &self,
    ) -> Vec<crate::domain_computation::primary_graph::WorthQueryApplicationProductBranchCleanup>
    {
        self.application
            .branches()
            .pending_cleanup()
            .expect("the authorization fixture keeps its owner open")
    }
}
