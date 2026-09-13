//! Producer branch-record work for exclusive output publication.
use super::{BranchMutationRecord, BranchStructuralDelta, SignalGraph};
use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::reuse::ReuseBasis;
use crate::logic::evaluation::EvaluationWork;

// set_causality, three record_effect_state_mutation writes, and snapshot commit.
// Direct downstream invalidation is a separate publication owner.
const PRODUCER_WRITES: usize = 5;

impl SignalGraph {
    pub(crate) fn admit_effect_branch_record_work(
        &self,
        node: NodeId,
        next_basis: Option<&ReuseBasis>,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        let previous = self.node_runtime_artifact_reuse_basis(node)?;
        // Initial structural-state/delta copies, two destination record copies,
        // and conservative copying after each preceding record append.
        for basis in [previous, next_basis].into_iter().flatten() {
            basis_copy(basis, 2 * (PRODUCER_WRITES + 2), work)?;
        }
        for records in [
            &self.observation.branch_mutation_view,
            &self.observation.branch_mutation_records,
        ] {
            let steps = records.lookup_steps();
            work.reserve(Some(steps))?;
            let existing = records.get(&node);
            // Each entry/or_default includes its search and possible insertion,
            // followed by get_mut. Fixed NodeId keys; permit retired readmission.
            work.reserve(
                steps
                    .checked_mul(32)
                    .and_then(|n| n.checked_mul(PRODUCER_WRITES)),
            )?;
            let count = existing.map_or(0, |record| record.structural_deltas.len());
            work.reserve(
                count
                    .checked_add(PRODUCER_WRITES)
                    .and_then(|n| n.checked_mul(std::mem::size_of::<BranchStructuralDelta>() + 1))
                    .and_then(|n| n.checked_mul(2 * PRODUCER_WRITES))
                    .filter(|n| *n <= isize::MAX as usize),
            )?;
            if let Some(record) = existing {
                record_copies(record, work)?;
            }
        }
        Ok(())
    }
}

fn record_copies(
    record: &BranchMutationRecord,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    for delta in &record.structural_deltas {
        match delta {
            BranchStructuralDelta::DependencyTopologyChanged(delta) => {
                for edges in [&delta.added_edges, &delta.removed_edges] {
                    work.reserve(
                        edges
                            .len()
                            .checked_mul(std::mem::size_of::<DependencyEdge>() + 1)
                            .and_then(|n| n.checked_mul(PRODUCER_WRITES))
                            .filter(|n| *n <= isize::MAX as usize),
                    )?;
                    for edge in edges {
                        let bytes = edge.scope_ref().map_or(Some(0), |scope| {
                            scope
                                .partition
                                .0
                                .len()
                                .checked_add(scope.detail.as_ref().map_or(0, String::len))
                        });
                        work.reserve(bytes.and_then(|n| n.checked_mul(PRODUCER_WRITES)))?;
                    }
                }
            }
            BranchStructuralDelta::RuntimeArtifactChanged(delta) => {
                for basis in [&delta.previous_reuse_basis, &delta.next_reuse_basis]
                    .into_iter()
                    .flatten()
                {
                    basis_copy(basis, PRODUCER_WRITES, work)?;
                }
            }
            BranchStructuralDelta::NodeIntroduced
            | BranchStructuralDelta::NodeStateChanged
            | BranchStructuralDelta::DependencySnapshotChanged(_)
            | BranchStructuralDelta::RetainedArtifactChanged
            | BranchStructuralDelta::CausalityChanged => {}
        }
    }
    Ok(())
}

fn basis_copy(
    basis: &ReuseBasis,
    copies: usize,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    let ReuseBasis {
        strategy: _,
        source: _,
        crossing: _,
        dependency_snapshot_basis: _,
        topology_regime_basis: _,
        structural_dependency_basis: _,
        artifact_family_basis,
        partition_region_basis_count: _,
    } = basis;
    work.reserve(
        artifact_family_basis
            .as_ref()
            .map_or(Some(1), |family| family.as_str().len().checked_add(1))
            .and_then(|n| n.checked_mul(copies)),
    )
}

#[cfg(test)]
mod tests;
