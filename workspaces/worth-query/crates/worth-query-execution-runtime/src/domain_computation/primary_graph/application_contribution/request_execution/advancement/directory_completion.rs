//! Host upkeep is triggered independently of the completed business meter.
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;
use crate::ports::HomeMaintenanceAfterAdvancement;

impl QueryRequestExecution<'_> {
    pub(super) fn run_product_advancement<R>(
        &self,
        product: &WorthQueryProductRuntime,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        // Advances the pulse once this advancement closes, if it published.
        let _publications = product.fields().commit_pulse.advancement_scope();
        let owner = Some(product.fields().owner.owner_identity());
        let source = Some(
            product
                .source()
                .authoritative_source_profile()
                .runtime_instance_id(),
        );
        self.run_advancement_with_maintenance(
            owner,
            source,
            product.home_maintenance(),
            retain_returned_outcome,
            body,
        )
    }
}
pub(super) fn after_business(
    request: ExecutionRequest<'_, '_>,
    maintenance: &dyn HomeMaintenanceAfterAdvancement,
) {
    // A completed caller may already be stopped. The closed Host owns upkeep's
    // independent controls and must not lose its trigger to that caller stop.
    maintenance.after_advancement(&request);
}
