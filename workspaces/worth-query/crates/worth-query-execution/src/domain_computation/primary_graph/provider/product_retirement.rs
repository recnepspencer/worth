//! Release Query-owned retention for one retired product occurrence.

use std::collections::BTreeSet;

use worth_runtime_world::facade::CompositeCommitIdentity;

use super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn has_outstanding_dispatch_for_branch(
        &self,
        incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> bool {
        self.outstanding_dispatch.has_branch_obligation(incarnation)
    }

    pub(in crate::domain_computation::primary_graph) fn settle_before_product_retirement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<(), crate::domain_computation::WorthQueryProviderSessionFailure> {
        self.graph
            .with_runtime_mut_unwind_isolated(execution, |runtime| {
                self.resume_pending_application_publication(execution, runtime, occurrence)
            })
    }

    pub(crate) fn release_product_occurrence_retention(
        &self,
        branch: &worth_runtime_world::facade::ProductBranchIdentity,
        incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        self.live_delivery
            .retire_product_occurrence(branch, incarnation);
        self.graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .release_occurrence(incarnation);
        // Evidence on a commit history still retains passes to the forks
        // that reach it; the rest goes with the occurrence.
        let published = self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .occurrence_commits(branch, incarnation);
        let worlds: Vec<_> = published.iter().map(|(_, world)| world.clone()).collect();
        let retained = self.retained_world_commits(&worlds);
        for (commit, world) in published {
            let released = {
                let mut evidence = self
                    .completed_commit_evidence
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if retained.contains(&world) {
                    evidence.inherit(commit);
                    None
                } else {
                    evidence.release(commit)
                }
            };
            self.receipt_basis_retention
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .release(commit);
            drop(released);
        }
        self.graph
            .release_workflow_instance_progress_for_branch(incarnation.ordinal());
        self.retire_application_branch_commit_lane(incarnation);
    }

    /// Releases inherited evidence whose World commit history let go; a
    /// branch cleanup calls it after releasing history.
    pub(in crate::domain_computation::primary_graph) fn release_unretained_inherited_evidence(
        &self,
    ) {
        let inherited = self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .inherited_commits();
        if inherited.is_empty() {
            return;
        }
        let retained = self.retained_world_commits(&inherited);
        let released: Vec<_> = {
            let mut evidence = self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inherited
                .iter()
                .filter(|world| !retained.contains(world))
                .filter_map(|world| evidence.release_inherited(world))
                .collect()
        };
        drop(released);
    }

    /// The World commits of `candidates` history still retains. A World that
    /// cannot answer keeps them all; nothing is lost but capacity.
    fn retained_world_commits(
        &self,
        candidates: &[CompositeCommitIdentity],
    ) -> BTreeSet<CompositeCommitIdentity> {
        let Some(world) = self.world_history.get() else {
            return BTreeSet::new();
        };
        world
            .retained_commits(candidates)
            .unwrap_or_else(|_| candidates.to_vec())
            .into_iter()
            .collect()
    }
}
