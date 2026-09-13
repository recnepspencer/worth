//! Materialized snapshot payload confined to the exclusive output packet.
use super::{ApplyCommitPacket, EvaluationWork, SignalError, SignalGraph};
use crate::data::dependency::{
    CommittedSnapshotUpdate, SnapshotDeltaRecord, SnapshotStorageStrategy,
};
use crate::data::handle::NodeId;

#[derive(Debug)]
pub(super) struct MaterializedEffectSnapshot {
    node: NodeId,
    insertion: super::snapshot_publication::PreparedEffectSnapshotStorage,
    delta: SnapshotDeltaRecord,
    strategy: SnapshotStorageStrategy,
}

impl SignalGraph {
    pub(super) fn materialize_effect_snapshot(
        &mut self,
        apply: &ApplyCommitPacket,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Option<MaterializedEffectSnapshot>, SignalError> {
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
                    let bytes = entry.scope.as_ref().map_or(Some(0), |s| {
                        s.partition
                            .0
                            .len()
                            .checked_add(s.detail.as_ref().map_or(0, String::len))
                    });
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
        let insertion =
            self.prepare_dependency_snapshot_insertion(snapshot.into_snapshot(), work)?;
        let insertion = self.prepare_effect_snapshot_storage(insertion, work)?;
        Ok(Some(MaterializedEffectSnapshot {
            node: effect.node,
            insertion,
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
