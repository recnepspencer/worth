//! Release Query-owned retention for one retired product occurrence.

use super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn settle_before_product_retirement(
        &self,
    ) -> Result<(), crate::domain_computation::WorthQueryProviderSessionFailure> {
        self.graph
            .with_runtime_mut(|runtime| self.resume_pending_application_publication(runtime))
    }

    pub(crate) fn release_product_occurrence_retention(
        &self,
        branch: &worth_runtime_world::facade::ProductBranchIdentity,
        incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        self.live_delivery
            .retire_product_occurrence(branch, incarnation);
        loop {
            let retired = self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take_one_for_product_occurrence(branch, incarnation);
            let Some((commit, evidence)) = retired else {
                break;
            };
            self.receipt_basis_retention
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .release(commit);
            drop(evidence);
        }
    }
}
