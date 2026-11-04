//! One installed route keeps the business value and original meter through upkeep.
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
    // A body can latch a joined or own stop without polling it. Only the
    // execution owner's completion guard may decide whether upkeep enters.
    // Its refusal never substitutes for the business callback's exact value.
    let _ = request.run_native_completion(|lease| {
        let invoking = lease.map(ExecutionRequest::leased).unwrap_or(request);
        maintenance.after_advancement(&invoking);
    });
}
