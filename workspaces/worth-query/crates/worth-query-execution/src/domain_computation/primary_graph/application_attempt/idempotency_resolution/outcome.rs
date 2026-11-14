//! Shared application outcome from the owner's authentic key resolution.
use super::*;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(in crate::domain_computation::primary_graph) fn envelope_resolution(
        &self,
        resolution: WorthQueryProviderIdempotencyResolution,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Result<
        WorthQueryApplicationIdempotencyResolution,
        WorthQueryApplicationIdempotencyResolutionDenial,
    > {
        use WorthQueryApplicationIdempotencyResolution as Outcome;
        use WorthQueryProviderIdempotencyResolution as Owner;
        Ok(match resolution {
            Owner::Readmitted(receipt) => Outcome::RecoveredCommitted(receipt),
            Owner::ReleasedOriginal(original) => Outcome::ReleasedOriginal(original),
            Owner::Absent => Outcome::Unseen,
            Owner::Drift => Outcome::IntentDrift,
            Owner::RecoveredUnpublished { handle } => Outcome::ProductUnpublished(
                self.runtime.primary_provider.recovered_unpublished_application(product, handle)
                    .map_err(|_| WorthQueryApplicationIdempotencyResolutionDenial::provider_unavailable())?
            ),
            Owner::PreparationPending { handle, stage } => Outcome::ProductPreparationPending(
                self.pending_preparation_resolution(handle, stage)
            ),
            Owner::PreparationBusy => return Err(WorthQueryApplicationIdempotencyResolutionDenial::from_provider(
                crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::PreparationBusy
            )),
            Owner::Unpublished => return Err(WorthQueryApplicationIdempotencyResolutionDenial::provider_unavailable()),
        })
    }
}
