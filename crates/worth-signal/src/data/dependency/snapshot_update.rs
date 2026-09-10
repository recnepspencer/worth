mod materialization;
mod replacement_preparation;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::data::handle::NodeId;

use super::{
    DependencySnapshot, DependencySnapshotId, SharedDependencySnapshot, SnapshotDeltaRecord,
    SnapshotShapeHandle, StableShapeSnapshotBasis, VersionOnlySnapshotUpdate, VersionVector,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplacementSnapshotUpdate {
    snapshot: SharedDependencySnapshot,
}

impl ReplacementSnapshotUpdate {
    pub(crate) fn from_snapshot(snapshot: DependencySnapshot) -> Self {
        Self::from_snapshot_with_work(
            snapshot,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .expect("ordinary snapshot preparation must remain representable")
    }

    pub fn snapshot(&self) -> &SharedDependencySnapshot {
        &self.snapshot
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommittedSnapshotUpdate {
    VersionOnly(VersionOnlySnapshotUpdate),
    Replace(ReplacementSnapshotUpdate),
}

impl CommittedSnapshotUpdate {
    pub fn between(
        node: NodeId,
        previous_snapshot_id: DependencySnapshotId,
        previous_shape_handle: SnapshotShapeHandle,
        previous: &DependencySnapshot,
        next: DependencySnapshot,
    ) -> (Self, SnapshotDeltaRecord) {
        let next = next.canonicalize_unordered();
        if snapshot_shape_matches(previous, &next) {
            let cached_versions = next
                .entries()
                .iter()
                .map(|entry| entry.cached_version)
                .collect::<Vec<_>>();
            let basis = StableShapeSnapshotBasis {
                node,
                previous_snapshot_id,
                shape_handle: previous_shape_handle,
                entry_count: cached_versions.len(),
            };
            let delta = SnapshotDeltaRecord::for_version_update(node, previous, &cached_versions);
            return (
                Self::VersionOnly(VersionOnlySnapshotUpdate::from_basis_and_versions(
                    basis,
                    VersionVector {
                        cached_versions: Arc::new(cached_versions),
                    },
                )),
                delta,
            );
        }

        let replacement = ReplacementSnapshotUpdate::from_snapshot(next);
        let delta = SnapshotDeltaRecord::between(node, previous, replacement.snapshot());
        (Self::Replace(replacement), delta)
    }

    pub fn storage_strategy(&self) -> SnapshotStorageStrategy {
        match self {
            Self::VersionOnly(_) => SnapshotStorageStrategy::VersionOnlyDelta,
            Self::Replace(_) => SnapshotStorageStrategy::SharedReplacement,
        }
    }

    pub fn entry_count(&self) -> usize {
        match self {
            Self::VersionOnly(update) => update.versions().len(),
            Self::Replace(update) => update.snapshot().entries().len(),
        }
    }

    pub fn apply_to(self, previous: &DependencySnapshot) -> SharedDependencySnapshot {
        match self {
            Self::VersionOnly(update) => SharedDependencySnapshot::new(
                previous.with_updated_versions(update.versions().as_slice()),
            ),
            Self::Replace(update) => update.snapshot,
        }
    }

    pub fn change_kind(&self) -> super::SnapshotChangeKind {
        match self {
            Self::VersionOnly(_) => super::SnapshotChangeKind::StableShapeVersionOnly,
            Self::Replace(_) => super::SnapshotChangeKind::StructuralReplace,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotStorageStrategy {
    SharedReplacement,
    VersionOnlyDelta,
}

fn snapshot_shape_matches(previous: &DependencySnapshot, next: &DependencySnapshot) -> bool {
    let previous_entries = previous.entries();
    let next_entries = next.entries();
    previous_entries.len() == next_entries.len()
        && previous_entries
            .iter()
            .zip(next_entries.iter())
            .all(|(left, right)| left.compare_key(right).is_eq())
}
