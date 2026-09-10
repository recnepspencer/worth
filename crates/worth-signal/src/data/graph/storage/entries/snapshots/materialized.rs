use super::*;
use crate::data::dependency::PreparedSnapshotInsertion;

impl SignalGraph {
    pub(crate) fn prepare_dependency_snapshot_insertion(
        &mut self,
        snapshot: DependencySnapshot,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<PreparedSnapshotInsertion, SignalError> {
        self.topology.dependency_snapshots.prepare_insertion(
            snapshot,
            &mut self.topology.dependency_snapshot_shapes,
            work,
        )
    }

    pub(crate) fn replace_dep_snapshot_materialized(
        &mut self,
        id: NodeId,
        insertion: PreparedSnapshotInsertion,
        delta: SnapshotDeltaRecord,
        strategy: SnapshotStorageStrategy,
    ) -> Result<SnapshotDeltaRecord, SignalError> {
        if delta.changed() {
            self.set_dep_snapshot_id_direct(id, insertion.snapshot_id())?;
        }
        Ok(self.publish_dependency_snapshot_storage(id, insertion, delta, strategy))
    }

    pub(in crate::data::graph) fn publish_dependency_snapshot_storage(
        &mut self,
        id: NodeId,
        insertion: PreparedSnapshotInsertion,
        delta: SnapshotDeltaRecord,
        strategy: SnapshotStorageStrategy,
    ) -> SnapshotDeltaRecord {
        if !delta.changed() {
            return self.record_dependency_snapshot_storage_publication(id, delta, strategy);
        }
        insertion.publish(
            &mut self.topology.dependency_snapshots,
            &mut self.topology.dependency_snapshot_shapes,
        );
        self.record_dependency_snapshot_storage_publication(id, delta, strategy)
    }

    pub(in crate::data::graph) fn record_dependency_snapshot_storage_publication(
        &mut self,
        id: NodeId,
        delta: SnapshotDeltaRecord,
        strategy: SnapshotStorageStrategy,
    ) -> SnapshotDeltaRecord {
        match strategy {
            SnapshotStorageStrategy::SharedReplacement => {
                self.with_telemetry(|telemetry| {
                    telemetry.storage.shared_snapshot_replacement_count += 1;
                    telemetry.storage.structural_replace_batch_commit_count += 1;
                });
            }
            SnapshotStorageStrategy::VersionOnlyDelta => {
                self.with_telemetry(|telemetry| {
                    telemetry.storage.version_only_snapshot_update_count += 1;
                    telemetry.storage.stable_shape_batch_commit_count += 1;
                    telemetry.storage.snapshot_shape_reuse_count += 1;
                });
            }
        }
        if delta.changed() {
            self.record_branch_mutation_snapshot(
                id,
                DependencySnapshotStructuralDelta::from_snapshot_delta(delta.clone()),
            );
            self.record_graph_storage_pressure();
        }
        delta
    }
}
