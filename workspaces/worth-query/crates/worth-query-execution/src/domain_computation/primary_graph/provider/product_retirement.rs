//! Release Query-owned retention for one retired product occurrence.

use std::collections::BTreeSet;

use worth_runtime_world::facade::{CompositeCommitIdentity, RuntimeWorldLifecyclePort};

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::execution_runtime::product_world::WorthQueryReleasedHistoryFollower;

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
        // that reach it; the rest goes with the occurrence. The World read and
        // the hand-over share one evidence lock: a cleanup that releases
        // history after the read sweeps once its walk ends, and that sweep
        // waits here and then finds the evidence inherited.
        let (commits, released): (Vec<_>, Vec<_>) = {
            let mut evidence = self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let published = evidence.occurrence_commits(branch, incarnation);
            let worlds: Vec<_> = published.iter().map(|(_, world)| world.clone()).collect();
            let retained = self.retained_world_commits(&worlds);
            published
                .into_iter()
                .map(|(commit, world)| {
                    let released = if retained.contains(&world) {
                        evidence.inherit(commit);
                        None
                    } else {
                        evidence.release(commit)
                    };
                    (commit, released)
                })
                .unzip()
        };
        {
            let mut bases = self
                .receipt_basis_retention
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for commit in commits {
                bases.release(commit);
            }
        }
        drop(released);
        self.graph
            .release_workflow_instance_progress_for_branch(incarnation.ordinal());
        self.retire_application_branch_commit_lane(incarnation);
    }

    /// Releases inherited evidence whose World commit history let go.
    fn release_unretained_inherited_evidence(&self) {
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

    fn retained_world_commits(
        &self,
        candidates: &[CompositeCommitIdentity],
    ) -> BTreeSet<CompositeCommitIdentity> {
        retained_among(self.world_history.get(), candidates)
    }
}

/// Every history release a branch or recovery cleanup makes reaches here.
impl WorthQueryReleasedHistoryFollower for WorthQueryPrimaryGraphProvider {
    fn history_released(&self) {
        self.release_unretained_inherited_evidence();
    }
}

/// The World commits of `candidates` history still retains. A World that is
/// not attached or cannot answer keeps them all; nothing is lost but capacity.
fn retained_among(
    world: Option<&RuntimeWorldLifecyclePort>,
    candidates: &[CompositeCommitIdentity],
) -> BTreeSet<CompositeCommitIdentity> {
    world
        .and_then(|world| world.retained_commits(candidates).ok())
        .unwrap_or_else(|| candidates.to_vec())
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::retained_among;
    use crate::domain_computation::primary_graph::tests::recoverable_commit_support;

    #[test]
    fn a_provider_without_world_history_keeps_every_candidate() {
        let receipt = recoverable_commit_support::committed_recoverable_application();
        let commit = receipt
            .committed_product_publication()
            .composite_commit()
            .clone();
        assert!(retained_among(None, std::slice::from_ref(&commit)).contains(&commit));
    }
}
