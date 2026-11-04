//! An installed request owner for provider-custody tests in sibling domains.
use crate::domain_computation::primary_graph::tests::fixture::*;
pub struct AdvancementHost(AuthorizationWorld);
impl AdvancementHost {
    pub fn placement(&self) -> worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'_> {
        self.0
            .application
            .runtime
            .product_runtime
            .owner
            .execution_placement()
    }
    pub fn install() -> Self {
        Self(installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()))
    }
}
