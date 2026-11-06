//! An installed request owner for provider-custody tests in sibling domains.
use crate::domain_computation::primary_graph::tests::fixture::*;
pub struct AdvancementHost(AuthorizationWorld);
impl AdvancementHost {
    pub fn with_placement<R>(
        &self,
        body: impl for<'owner> FnOnce(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'owner>,
        ) -> R,
    ) -> R {
        self.0
            .application
            .product_runtime()
            .with_execution_placement_for_test(body)
            .expect("the installed fixture World remains available")
    }
    pub fn install() -> Self {
        Self(installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()))
    }
}
