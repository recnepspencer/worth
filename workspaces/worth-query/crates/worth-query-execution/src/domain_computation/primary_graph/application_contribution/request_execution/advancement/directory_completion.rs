//! Host upkeep is triggered independently of the completed business meter.
use super::*;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryHomeMaintenance, WorthQueryProductRuntime,
};

impl QueryRequestExecution<'_> {
    pub(super) fn run_product_advancement<R>(
        &self,
        product: &WorthQueryProductRuntime,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let owner = Some(product.owner.owner_identity());
        let source = Some(
            product
                .source()
                .authoritative_source_profile()
                .runtime_instance_id(),
        );
        match product.home_maintenance() {
            Some(maintenance) => {
                self.run_advancement_with_maintenance(owner, source, Some(maintenance), body)
            }
            None => self.run_advancement(owner, source, body),
        }
    }
}
pub(super) fn after_business(
    request: ExecutionRequest<'_, '_>,
    maintenance: &dyn WorthQueryHomeMaintenance,
) {
    // A completed caller may already be stopped. The closed Host owns upkeep's
    // independent controls and must not lose its trigger to that caller stop.
    maintenance.after_advancement(&request);
}
