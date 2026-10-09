//! An installed request owner for provider-custody tests in sibling domains.
use super::*;
pub(crate) struct AdvancementHost(AuthorizationWorld);
impl AdvancementHost {
    pub(crate) fn placement(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'_> {
        self.0
            .application
            .product_runtime
            .owner
            .execution_placement()
    }
    pub(crate) fn install() -> Self {
        Self(installed_authorization_world(true))
    }
}
