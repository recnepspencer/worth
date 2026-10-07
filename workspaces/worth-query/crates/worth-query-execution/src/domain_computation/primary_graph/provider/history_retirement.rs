//! History retirement behind a settled application publication.

use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_world::facade::{CompositeCommitIdentity, RuntimeWorldLifecyclePort};

use super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn install_world_history(
        &self,
        world: RuntimeWorldLifecyclePort,
    ) {
        assert!(
            self.world_history.set(world).is_ok(),
            "one product World owns the provider's history"
        );
    }

    /// World retires every commit behind `settled` that nothing protects,
    /// splicing retired commits out from between kept ones, and the receipt
    /// bases of those commits go with it, so inspection of a retired commit
    /// is unavailable. Their completed evidence keeps only the
    /// publication's identities, so replay still answers. Relational then
    /// reclaims the roots the retired history no longer pins. A World that
    /// cannot retire keeps its history; nothing is lost but capacity.
    pub(super) fn retire_history_behind(
        &self,
        runtime: &mut RelationalRuntime,
        settled: &CompositeCommitIdentity,
    ) {
        let retired = self
            .world_history
            .get()
            .and_then(|world| world.retire_unprotected_history(settled).ok())
            .unwrap_or_default();
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
        runtime.run_index_generation_reclamation_pass();
    }
}

#[cfg(feature = "test-query-execution-observer")]
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// What this product's history retains now: installed World commits,
    /// World history metadata bytes, unique World component pins and
    /// Relational retired branch roots awaiting reclamation.
    #[doc(hidden)]
    pub fn history_retained_for_test(
        &self,
    ) -> Result<(usize, usize, usize, usize), crate::facade::primary_graph::WorthQueryHandleDenial>
    {
        let retired_roots = self
            .primary_provider
            .graph
            .with_runtime(|runtime| runtime.retired_branch_root_count())?;
        let world = self.product_runtime.owner.inspection_port();
        let history = world
            .history_snapshot()
            .expect("an installed product World is available");
        let retention = world
            .retention_snapshot()
            .expect("an installed product World is available");
        Ok((
            history.installed_commits(),
            history.metadata().total_occupancy(),
            retention.unique_pins(),
            retired_roots,
        ))
    }

    /// Completed evidence entries inside the idempotency window and the bytes
    /// their tickets hold.
    #[doc(hidden)]
    pub fn completed_evidence_retained_for_test(&self) -> (usize, usize) {
        self.primary_provider.completed_evidence_retained()
    }
}
