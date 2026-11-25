//! History retirement behind a settled application publication.

use std::sync::Arc;

use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_relational::facade::runtime::RelationalRuntime;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

impl WorthQueryPrimaryGraphProvider {
    /// The provider reads one product World's history and follows every
    /// history release that product's branch and recovery cleanups make.
    pub fn install_world_history(self: &Arc<Self>, product: &WorthQueryProductRuntime) {
        assert!(
            self.world_history
                .set(product.owner_ref().recovery_port())
                .is_ok(),
            "one product World owns the provider's history"
        );
        let follower: std::sync::Weak<Self> = Arc::downgrade(self);
        product
            .owner_cleanup_ref()
            .follow_product_retention(follower);
    }

    /// World retires changed history interiors that nothing protects,
    /// splicing retired commits out from between kept ones, and the receipt
    /// bases of those commits go with it, so inspection of a retired commit
    /// is unavailable. Their completed evidence keeps only the
    /// publication's identities, so replay still answers. Relational then
    /// reclaims the roots the retired history no longer pins. A World that
    /// cannot retire keeps its history; nothing is lost but capacity.
    pub fn retire_changed_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        committed: &worth_relational::facade::transactions::CommitResult,
    ) {
        let retired = match self
            .world_history
            .get()
            .map(|world| world.retire_changed_history_with_request(execution))
        {
            Some(Ok(retired)) => retired,
            Some(Err(
                worth_runtime_world::facade::HistoryReclamationDenial::PartialPublication {
                    reclaimed,
                    ..
                },
            )) => reclaimed,
            Some(Err(_)) | None => Vec::new(),
        };
        if !retired.is_empty() {
            let commits = self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .retire(&retired);
            let mut bases = self
                .receipt_basis_retention
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for commit in commits {
                bases.release_ordinary(commit);
            }
        }
        runtime.run_branch_root_reclamation_pass();
        let _ = runtime.run_index_generation_reclamation_after_publication(committed);
    }
}
