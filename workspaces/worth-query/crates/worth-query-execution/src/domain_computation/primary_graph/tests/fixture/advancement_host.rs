//! An installed request owner for provider-custody tests in sibling domains.
use super::*;
pub(crate) struct AdvancementHost(AuthorizationWorld);
impl AdvancementHost {
    pub(crate) fn with_placement<R>(
        &self,
        body: impl for<'owner> FnOnce(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'owner>,
        ) -> R,
    ) -> R {
        self.0
            .application
            .product_runtime
            .with_execution_placement(body)
            .expect("the installed fixture World remains available")
    }
    pub(crate) fn install() -> Self {
        Self(installed_authorization_world(true))
    }
}
