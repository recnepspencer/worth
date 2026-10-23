//! Private, read-only selection of retained correction evidence by the actual native key.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

pub(in crate::domain_computation::primary_graph) enum CorrectionIdempotencyObservation {
    Unseen,
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
