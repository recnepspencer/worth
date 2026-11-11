//! Claim observation requires a fresh admitted original read, never a forged live receipt.
use super::WorthQueryRecoveryClaimStatus;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
    WorthQueryApplicationIdempotencyResolutionDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Observes only this owner's existing claim. It neither mints a handle nor
    /// grants dispatch authority. An unseen or changed key has no claim result.
    pub fn resolve_admitted_recovery_claim_status<Operation, Input, Scope>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryRecoveryClaimStatus>,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        let read = self.resolve_admitted_application_idempotency(phase, admission, binding)?;
        match read.into_resolution() {
            WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(original) => {
                let (claimed, terminal) = self.runtime.recovery_handles.recorded_claim_state(
                    self.runtime
                        .primary_provider
                        .graph
                        .relational_runtime_instance_id(),
                    original.new_product_head(),
                );
                Ok(Some(super::mint::status_from_claim_state(
                    claimed, terminal,
                )))
            }
            WorthQueryApplicationIdempotencyResolution::ReleasedOriginal(_) => Ok(None),
            WorthQueryApplicationIdempotencyResolution::ProductUnpublished(_)
            | WorthQueryApplicationIdempotencyResolution::ProductPreparationPending(_)
            | WorthQueryApplicationIdempotencyResolution::Unseen
            | WorthQueryApplicationIdempotencyResolution::IntentDrift => Ok(None),
        }
    }
}
