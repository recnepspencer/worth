mod materialized;
use crate::clock::RuntimeInstant;
use crate::data::dependency::{
    CommittedSnapshotUpdate, DependencySnapshot, SnapshotDeltaRecord, SnapshotStorageStrategy,
};
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::{DependencySnapshotStructuralDelta, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::proof::{
    ClassifiedSnapshotBatchCommit, MixedSnapshotBatchCommit, PendingSnapshotBatch,
    SnapshotBatchCommit, StableShapeSnapshotBatchCommit,
};

impl SignalGraph {
    pub(crate) fn get_dep_snapshot(&self, id: NodeId) -> Result<&DependencySnapshot, SignalError> {
        Ok(self
            .topology
            .dependency_snapshots
            .get(self.hot_ref(id)?.dep_snapshot_id))
    }

    pub(crate) fn dependency_snapshot_shape_handle(
        &mut self,
        id: crate::data::dependency::DependencySnapshotId,
    ) -> crate::data::dependency::SnapshotShapeHandle {
        self.topology
            .dependency_snapshots
            .shape_handle_for(id, &mut self.topology.dependency_snapshot_shapes)
    }

    pub(crate) fn dependency_snapshot_shape_handle_for_evaluation(
        &mut self,
        id: crate::data::dependency::DependencySnapshotId,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<crate::data::dependency::SnapshotShapeHandle, SignalError> {
        match work {
            crate::logic::evaluation::EvaluationWork::Ordinary => {
                Ok(self.dependency_snapshot_shape_handle(id))
            }
            crate::logic::evaluation::EvaluationWork::Conditional(_) => {
                work.reserve(Some(
                    self.topology
                        .dependency_snapshots
                        .retained_shape_handle_lookup_steps(),
                ))?;
                self.topology
                    .dependency_snapshots
                    .retained_shape_handle_for(id, &self.topology.dependency_snapshot_shapes)
                    .map_err(|_| SignalError::SnapshotIndexUnavailable)
            }
        }
    }

    fn insert_dependency_snapshot(
        &mut self,
        snapshot: DependencySnapshot,
    ) -> crate::data::dependency::DependencySnapshotId {
        self.topology
            .dependency_snapshots
            .insert_with_shape_handle(snapshot, &mut self.topology.dependency_snapshot_shapes)
            .0
    }

    pub(crate) fn set_dep_snapshot(
        &mut self,
        id: NodeId,
        snapshot: DependencySnapshot,
    ) -> Result<(), SignalError> {
        let previous = self.get_dep_snapshot(id)?.clone();
        let (_, previous_snapshot_id) = self.node_dependency_ids(id)?;
        let previous_shape_handle = self.dependency_snapshot_shape_handle(previous_snapshot_id);
        let (update, delta) = CommittedSnapshotUpdate::between(
            id,
            previous_snapshot_id,
            previous_shape_handle,
            &previous,
            snapshot,
        );
        if !delta.changed() {
            return Ok(());
        }
        match update.storage_strategy() {
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
        let snapshot_id =
            self.insert_dependency_snapshot(update.apply_to(&previous).into_snapshot());
        self.set_dep_snapshot_id_direct(id, snapshot_id)?;
        self.record_branch_mutation_snapshot(
            id,
            DependencySnapshotStructuralDelta::from_snapshot_delta(delta),
        );
        self.record_graph_storage_pressure();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn replace_dep_snapshot_committed(
        &mut self,
        id: NodeId,
        update: CommittedSnapshotUpdate,
    ) -> Result<SnapshotDeltaRecord, SignalError> {
        let previous = self.get_dep_snapshot(id)?;
        let delta = match &update {
            CommittedSnapshotUpdate::VersionOnly(version) => {
                SnapshotDeltaRecord::for_version_update(id, previous, version.versions().as_slice())
            }
            CommittedSnapshotUpdate::Replace(replacement) => {
                SnapshotDeltaRecord::between(id, previous, replacement.snapshot())
            }
        };
        let snapshot = update.materialize_with_work(
            previous,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )?;
        let insertion = self.prepare_dependency_snapshot_insertion(
            snapshot.into_snapshot(),
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )?;
        self.replace_dep_snapshot_materialized(id, insertion, delta, update.storage_strategy())
    }

    pub(crate) fn apply_stable_shape_snapshot_batch_commit(
        &mut self,
        commit: StableShapeSnapshotBatchCommit,
    ) -> Result<(), SignalError> {
        if commit.is_empty() {
            return Ok(());
        }
        let batch_size = commit.pending().len() as u64;
        self.with_telemetry(|telemetry| {
            telemetry.storage.snapshot_batch_size += batch_size;
            telemetry.storage.stable_shape_batch_commit_count += 1;
        });

        for snapshot in commit.pending() {
            self.validate_handle(snapshot.node())?;
        }

        for snapshot in commit.pending() {
            if !snapshot.delta().changed() {
                continue;
            }
            self.with_telemetry(|telemetry| {
                telemetry.storage.patch_application_breadth += 1;
                telemetry.storage.version_only_snapshot_update_count += 1;
                telemetry.storage.snapshot_shape_reuse_count += 1;
            });
            let previous = self.get_dep_snapshot(snapshot.node())?.clone();
            let next_snapshot = CommittedSnapshotUpdate::VersionOnly(snapshot.update().clone())
                .apply_to(&previous)
                .into_snapshot();
            let snapshot_id = self.insert_dependency_snapshot(next_snapshot);
            self.set_dep_snapshot_id_direct(snapshot.node(), snapshot_id)?;
            self.record_branch_mutation_snapshot(
                snapshot.node(),
                DependencySnapshotStructuralDelta::from_snapshot_delta(snapshot.delta()),
            );
        }

        self.record_graph_storage_pressure();
        Ok(())
    }

    pub(crate) fn apply_mixed_snapshot_batch_commit(
        &mut self,
        commit: MixedSnapshotBatchCommit,
    ) -> Result<(), SignalError> {
        if commit.is_empty() {
            return Ok(());
        }
        let batch_size = (commit.stable_shape().len() + commit.replacements().len()) as u64;
        self.with_telemetry(|telemetry| {
            telemetry.storage.snapshot_batch_size += batch_size;
            telemetry.storage.structural_replace_batch_commit_count += 1;
        });

        for snapshot in commit.stable_shape() {
            self.validate_handle(snapshot.node())?;
        }
        for snapshot in commit.replacements() {
            self.validate_handle(snapshot.node())?;
        }

        for snapshot in commit.stable_shape() {
            if !snapshot.delta().changed() {
                continue;
            }
            self.with_telemetry(|telemetry| {
                telemetry.storage.patch_application_breadth += 1;
                telemetry.storage.version_only_snapshot_update_count += 1;
                telemetry.storage.snapshot_shape_reuse_count += 1;
            });
            let previous = self.get_dep_snapshot(snapshot.node())?.clone();
            let next_snapshot = CommittedSnapshotUpdate::VersionOnly(snapshot.update().clone())
                .apply_to(&previous)
                .into_snapshot();
            let snapshot_id = self.insert_dependency_snapshot(next_snapshot);
            self.set_dep_snapshot_id_direct(snapshot.node(), snapshot_id)?;
            self.record_branch_mutation_snapshot(
                snapshot.node(),
                DependencySnapshotStructuralDelta::from_snapshot_delta(snapshot.delta()),
            );
        }

        for snapshot in commit.replacements() {
            if !snapshot.delta().changed() {
                continue;
            }
            self.with_telemetry(|telemetry| {
                telemetry.storage.patch_application_breadth += 1;
                telemetry.storage.shared_snapshot_replacement_count += 1;
            });
            let next_snapshot = CommittedSnapshotUpdate::Replace(snapshot.update().clone())
                .apply_to(self.get_dep_snapshot(snapshot.node())?)
                .into_snapshot();
            let snapshot_id = self.insert_dependency_snapshot(next_snapshot);
            self.set_dep_snapshot_id_direct(snapshot.node(), snapshot_id)?;
            self.record_branch_mutation_snapshot(
                snapshot.node(),
                DependencySnapshotStructuralDelta::from_snapshot_delta(snapshot.delta()),
            );
        }

        self.record_graph_storage_pressure();
        Ok(())
    }

    pub(crate) fn apply_classified_snapshot_batch_commit(
        &mut self,
        commit: ClassifiedSnapshotBatchCommit,
    ) -> Result<(), SignalError> {
        let commit_start = self
            .captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
            )
            .then(RuntimeInstant::now);
        let result = match commit {
            ClassifiedSnapshotBatchCommit::StableShape(commit) => {
                self.apply_stable_shape_snapshot_batch_commit(commit)
            }
            ClassifiedSnapshotBatchCommit::Mixed(commit) => {
                self.apply_mixed_snapshot_batch_commit(commit)
            }
        };
        if let Some(commit_start) = commit_start {
            let elapsed_nanos = commit_start.elapsed().as_nanos();
            self.with_telemetry(|telemetry| {
                telemetry.storage.snapshot_batch_commit_nanos += elapsed_nanos;
            });
        }
        result
    }

    #[allow(dead_code)]
    pub(crate) fn derive_dependency_snapshot_restore_batch(
        &self,
        target: &SignalGraph,
    ) -> Result<SnapshotBatchCommit, SignalError> {
        let mut entries = Vec::new();
        for index in 0..target.arena_capacity() {
            let Some(node) = target.live_node_id_at(index) else {
                continue;
            };
            if !self.is_alive(node) {
                continue;
            }
            let previous = self.get_dep_snapshot(node)?.clone();
            let next = target.get_dep_snapshot(node)?.clone();
            let (_, previous_snapshot_id) = self.node_dependency_ids(node)?;
            let mut shape_store = self.topology.dependency_snapshot_shapes.clone();
            let previous_shape_handle = previous.shape().intern(&mut shape_store);
            let (update, delta) = CommittedSnapshotUpdate::between(
                node,
                previous_snapshot_id,
                previous_shape_handle,
                &previous,
                next,
            );
            if delta.changed() {
                entries.push(crate::data::proof::PendingSnapshotCommit {
                    node,
                    update,
                    delta,
                });
            }
        }
        Ok(SnapshotBatchCommit::new(PendingSnapshotBatch::new(entries)))
    }
}

#[cfg(test)]
mod lookup_work_tests;
