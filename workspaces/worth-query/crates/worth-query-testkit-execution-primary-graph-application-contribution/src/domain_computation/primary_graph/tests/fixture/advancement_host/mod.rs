//! An installed request owner for provider-custody tests in sibling domains.
use crate::domain_computation::primary_graph::tests::fixture::*;
pub struct AdvancementHost(AuthorizationWorld);
impl AdvancementHost {
    pub fn placement(&self) -> worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'_> {(*
        (*self.0
            .application
            .product_runtime_for_test())
            .owner_for_test())
            .execution_placement()
    }
    pub fn install() -> Self {
        Self(installed_authorization_world(true))
    }
}
