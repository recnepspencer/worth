//! Materialized snapshot payload confined to the exclusive output packet.
use super::{ApplyCommitPacket, EvaluationWork, SignalError, SignalGraph};
use crate::data::dependency::{
    CommittedSnapshotUpdate, SnapshotDeltaRecord, SnapshotStorageStrategy,
};
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;

#[derive(Debug)]
pub(super) struct MaterializedEffectSnapshot {
    node: NodeId,
    insertion: super::snapshot_publication::PreparedEffectSnapshotStorage,
    delta: SnapshotDeltaRecord,
    strategy: SnapshotStorageStrategy,
}

pub(super) struct PreparedSnapshotCandidate {
    pub(super) node: NodeId,
    pub(super) snapshot: crate::data::dependency::DependencySnapshot,
    pub(super) delta: SnapshotDeltaRecord,
    pub(super) strategy: SnapshotStorageStrategy,
}

impl SignalGraph {
    pub(super) fn claim_epoch_snapshot_candidate_shape(
        &self,
        apply: &ApplyCommitPacket,
        preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<(), SignalError> {
        let Some(budget) = preparation else {
            return Ok(());
        };
        let effect = &apply.effect.operational;
        if apply.defer_snapshot_commit
            || !effect.snapshot_delta.changed()
            || !super::super::vocabulary::verdict_commits_snapshot(&effect.verdict)
        {
            return Ok(());
        }
        if let CommittedSnapshotUpdate::VersionOnly(_) = &effect.dependency_snapshot_update {
            let previous = self.get_dep_snapshot(effect.node)?;
            // Arc::make_mut duplicates the shared entry vector and each
            // scoped entry before changing versions.
            budget.claim_vec::<crate::data::dependency::DependencySnapshotEntry>(
                previous.entries().len(),
            )?;
            budget.claim_vec::<usize>(5)?;
            for entry in previous.entries() {
                if let Some(scope) = entry.scope.as_ref() {
                    budget.claim_vec::<String>(scope.path().depth())?;
                    budget.claim_vec::<u8>(scope.path().total_segment_bytes())?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn materialize_effect_snapshot(
        &mut self,
        apply: &ApplyCommitPacket,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<Option<MaterializedEffectSnapshot>, SignalError> {
        let Some(candidate) = self.prepare_effect_snapshot_candidate(apply, work)? else {
            return Ok(None);
        };
        let insertion = self.prepare_dependency_snapshot_insertion(candidate.snapshot, work)?;
        let insertion = self.prepare_effect_snapshot_storage(insertion, work)?;
        Ok(Some(MaterializedEffectSnapshot {
            node: candidate.node,
            insertion,
            delta: candidate.delta,
            strategy: candidate.strategy,
        }))
    }

    pub(super) fn prepare_effect_snapshot_candidate(
        &self,
        apply: &ApplyCommitPacket,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<Option<PreparedSnapshotCandidate>, SignalError> {
        let effect = &apply.effect.operational;
        if apply.defer_snapshot_commit
            || !effect.snapshot_delta.changed()
            || !super::super::vocabulary::verdict_commits_snapshot(&effect.verdict)
        {
            return Ok(None);
        }
        let previous = self.get_dep_snapshot(effect.node)?;
        let update = &effect.dependency_snapshot_update;
        let delta = match update {
            CommittedSnapshotUpdate::VersionOnly(version) => {
                if previous.entries().len() != version.versions().len() {
                    return Err(SignalError::invalid_input(
                        "snapshot version count does not match its shape",
                    ));
                }
                work.reserve(
                    previous
                        .entries()
                        .len()
                        .checked_mul(2)
                        .and_then(|n| n.checked_add(1)),
                )?;
                SnapshotDeltaRecord::for_version_update(
                    effect.node,
                    previous,
                    version.versions().as_slice(),
                )
            }
            CommittedSnapshotUpdate::Replace(replacement) => {
                let left = previous.entries();
                let right = replacement.snapshot().entries();
                let count = left
                    .len()
                    .checked_add(right.len())
                    .filter(|n| *n <= u32::MAX as usize);
                work.reserve(count)?;
                let mut largest = 0;
                for entry in left.iter().chain(right) {
                    let bytes = entry
                        .scope
                        .as_ref()
                        .map_or(Some(0), |s| s.path().checked_segment_bytes());
                    work.reserve(bytes.map(|_| 0))?;
                    largest = largest.max(bytes.expect("admitted scope length"));
                }
                work.reserve(
                    count.and_then(|n| n.checked_mul(largest.checked_mul(2)?.checked_add(16)?)),
                )?;
                SnapshotDeltaRecord::between(effect.node, previous, replacement.snapshot())
            }
        };
        let snapshot = update.materialize_with_work(previous, work)?;
        Ok(Some(PreparedSnapshotCandidate {
            node: effect.node,
            snapshot: snapshot.into_snapshot(),
            delta,
            strategy: update.storage_strategy(),
        }))
    }
}

impl MaterializedEffectSnapshot {
    pub(super) fn node_snapshot_id(&self) -> Option<crate::data::dependency::DependencySnapshotId> {
        self.delta.changed().then(|| self.insertion.snapshot_id())
    }

    pub(super) fn publish(self, graph: &mut SignalGraph) {
        graph.publish_effect_snapshot_storage(self.node, self.insertion, self.delta, self.strategy);
    }
}

#[cfg(test)]
mod tests;
