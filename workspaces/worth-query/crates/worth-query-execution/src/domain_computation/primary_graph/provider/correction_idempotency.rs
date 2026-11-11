//! Private, read-only selection of retained correction evidence by the actual native key.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

pub enum CorrectionIdempotencyObservation {
    Unseen,
    Recorded(super::idempotency::snapshot_resolution::WorthQueryRecoveredApplicationCommitReceipt),
    Completed(
        WorthQueryApplicationIdempotencyBinding,
        WorthQueryPrimaryGraphCommittedApplication,
    ),
    Pending(Option<WorthQueryApplicationIdempotencyBinding>),
    Unpublished(
        WorthQueryApplicationIdempotencyBinding,
        worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
    ),
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn observe_correction_command(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        key: &[u8; 32],
    ) -> Result<
        CorrectionIdempotencyObservation,
        super::idempotency::WorthQueryProviderIdempotencyResolutionDenial,
    > {
        if product.observation().owner_identity().durable().is_none() {
            return Ok(self.observe_correction_key(product.observation(), key));
        }
        if let Some(receipt) =
            super::idempotency::snapshot_resolution::observe(execution, self, product, *key)?
        {
            return Ok(CorrectionIdempotencyObservation::Recorded(receipt));
        }
        Ok(self.observe_correction_key(product.observation(), key))
    }
    /// Observation never resumes pending publication or claims a recovery handle.
    pub(in crate::domain_computation::primary_graph) fn observe_correction_key(
        &self,
        product: &worth_runtime_world::facade::ProductBranchObservation,
        key: &[u8; 32],
    ) -> CorrectionIdempotencyObservation {
        let command_lane = self.correction_key_lane(key);
        let _command = command_lane.enter();
        let lane = self.application_branch_commit_lane(product);
        let _coordination = lane.enter();
        let lookup = super::idempotency::WorthQueryIdempotencyLookupKey::Correction(*key);
        if let Some((binding, committed)) = self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe_lookup_key(&lookup)
        {
            return CorrectionIdempotencyObservation::Completed(binding, committed);
        }
        if let Some((binding, handle)) = self.inspect_unpublished_key(&lookup) {
            return CorrectionIdempotencyObservation::Unpublished(binding, handle);
        }
        if let Some(binding) = self.inspect_pending_correction_idempotency(key) {
            return CorrectionIdempotencyObservation::Pending(Some(binding));
        }
        if let Some(binding) =
            self.inspect_pending_application_idempotency(product.lifecycle_incarnation())
        {
            return CorrectionIdempotencyObservation::Pending(binding);
        }
        CorrectionIdempotencyObservation::Unseen
    }
}

mod court_doors_for_test;
