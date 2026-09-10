mod shape_materialization;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

mod normalization;
mod retained_charge;

use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;

use super::{DependencySnapshotId, DependencySnapshotShape, SnapshotShapeHandle};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DependencySnapshotEntry {
    pub source: NodeId,
    pub aspect: crate::data::aspect::Aspect,
    pub cached_version: u64,
    #[serde(default)]
    pub scope: Option<PartitionSubscription>,
}

impl DependencySnapshotEntry {
    pub(crate) fn compare_key(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.source.index(),
            self.source.generation(),
            self.aspect.index(),
            &self.scope,
        )
            .cmp(&(
                other.source.index(),
                other.source.generation(),
                other.aspect.index(),
                &other.scope,
            ))
    }

    pub(crate) fn compare_dependency(&self, other: &super::DependencyEdge) -> std::cmp::Ordering {
        (
            self.source.index(),
            self.source.generation(),
            self.aspect.index(),
            self.scope.as_ref(),
        )
            .cmp(&(
                other.source().index(),
                other.source().generation(),
                other.aspect().index(),
                other.scope_ref(),
            ))
    }

    pub fn sort_key(&self) -> super::DependencySortKey {
        super::DependencySortKey {
            source_index: self.source.index(),
            source_generation: self.source.generation(),
            aspect_index: self.aspect.index(),
            scope: self.scope.clone(),
        }
    }
}

/// A snapshot of upstream aspect versions at the time a node was last evaluated.
///
/// Used by the pull phase to determine if a `MaybeStale` node can revert
/// to `Clean`: if all upstream versions match the snapshot, no recomputation needed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DependencySnapshot {
    entries: Arc<Vec<DependencySnapshotEntry>>,
}

impl DependencySnapshot {
    /// Create an empty snapshot.
    pub fn empty() -> Self {
        Self {
            entries: Arc::new(Vec::new()),
        }
    }

    /// Record an upstream version.
    pub fn record(
        &mut self,
        source: NodeId,
        aspect: crate::data::aspect::Aspect,
        version: u64,
        scope: Option<PartitionSubscription>,
    ) {
        Arc::make_mut(&mut self.entries).push(DependencySnapshotEntry {
            source,
            aspect,
            cached_version: version,
            scope,
        });
    }

    /// All recorded entries.
    pub fn entries(&self) -> &[DependencySnapshotEntry] {
        self.entries.as_slice()
    }

    pub fn canonicalize_unordered(mut self) -> Self {
        self.canonicalize_in_place();
        self
    }

    pub(crate) fn canonicalize_with_work(
        mut self,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<Self, crate::data::error::SignalError> {
        normalization::normalize(&mut self, work)?;
        Ok(self)
    }

    pub fn from_ordered_unique(entries: impl IntoIterator<Item = DependencySnapshotEntry>) -> Self {
        let entries = entries.into_iter().collect::<Vec<_>>();
        debug_assert!(is_strict_snapshot_entry_order(entries.as_slice()));
        Self {
            entries: Arc::new(entries),
        }
    }

    fn canonicalize_in_place(&mut self) {
        normalization::normalize(
            self,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .expect("ordinary snapshot normalization must remain representable");
    }

    pub fn shared_entries(&self) -> Arc<Vec<DependencySnapshotEntry>> {
        Arc::clone(&self.entries)
    }

    pub fn shape(&self) -> DependencySnapshotShape {
        DependencySnapshotShape::from_ordered_unique(
            self.entries().iter().map(DependencySnapshotEntry::sort_key),
        )
    }

    /// Whether two snapshots currently share the same backing storage.
    ///
    /// This is a storage fact only. Snapshot identity, restore semantics, and
    /// reuse semantics must still be defined by explicit snapshot contracts.
    pub fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entries, &other.entries)
    }

    pub fn with_updated_versions(&self, cached_versions: &[u64]) -> Self {
        debug_assert_eq!(self.entries().len(), cached_versions.len());
        let mut updated = self.clone();
        for (entry, cached_version) in Arc::make_mut(&mut updated.entries)
            .iter_mut()
            .zip(cached_versions.iter().copied())
        {
            entry.cached_version = cached_version;
        }
        updated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotChangeKind {
    Unchanged,
    StableShapeVersionOnly,
    StructuralReplace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyInputScan {
    node: NodeId,
    previous_snapshot_id: DependencySnapshotId,
    previous_entry_count: usize,
    ordered_dependency_count: usize,
    shape_stable: bool,
    stable_shape_versions: Arc<Vec<u64>>,
}

impl DependencyInputScan {
    pub(crate) fn stable_shape(
        node: NodeId,
        previous_snapshot_id: DependencySnapshotId,
        previous_entry_count: usize,
        ordered_dependency_count: usize,
        stable_shape_versions: Vec<u64>,
    ) -> Self {
        Self {
            node,
            previous_snapshot_id,
            previous_entry_count,
            ordered_dependency_count,
            shape_stable: true,
            stable_shape_versions: Arc::new(stable_shape_versions),
        }
    }

    pub(crate) fn stable_shape_versions(&self) -> &[u64] {
        self.stable_shape_versions.as_slice()
    }

    pub(crate) fn stable_shape_versions_arc(&self) -> Arc<Vec<u64>> {
        Arc::clone(&self.stable_shape_versions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StableShapeSnapshotBasis {
    pub(super) node: NodeId,
    pub(super) previous_snapshot_id: DependencySnapshotId,
    pub(super) shape_handle: SnapshotShapeHandle,
    pub(super) entry_count: usize,
}

impl StableShapeSnapshotBasis {
    pub(crate) fn prove(
        scan: &DependencyInputScan,
        previous_shape_handle: SnapshotShapeHandle,
    ) -> Option<Self> {
        if !scan.shape_stable {
            return None;
        }
        if scan.previous_entry_count != scan.ordered_dependency_count {
            return None;
        }
        Some(Self {
            node: scan.node,
            previous_snapshot_id: scan.previous_snapshot_id,
            shape_handle: previous_shape_handle,
            entry_count: scan.ordered_dependency_count,
        })
    }

    #[cfg(test)]
    pub(crate) fn shape_handle(&self) -> SnapshotShapeHandle {
        self.shape_handle
    }

    pub(crate) fn entry_count(&self) -> usize {
        self.entry_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionVector {
    pub(super) cached_versions: Arc<Vec<u64>>,
}

impl VersionVector {
    pub(crate) fn from_scan(basis: &StableShapeSnapshotBasis, scan: &DependencyInputScan) -> Self {
        debug_assert_eq!(basis.entry_count(), scan.stable_shape_versions().len());
        Self {
            cached_versions: scan.stable_shape_versions_arc(),
        }
    }

    pub fn as_slice(&self) -> &[u64] {
        self.cached_versions.as_slice()
    }

    pub fn len(&self) -> usize {
        self.cached_versions.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionOnlySnapshotUpdate {
    basis: StableShapeSnapshotBasis,
    versions: VersionVector,
}

impl VersionOnlySnapshotUpdate {
    pub(crate) fn from_basis_and_versions(
        basis: StableShapeSnapshotBasis,
        versions: VersionVector,
    ) -> Self {
        Self { basis, versions }
    }

    pub fn basis(&self) -> &StableShapeSnapshotBasis {
        &self.basis
    }

    pub fn versions(&self) -> &VersionVector {
        &self.versions
    }
}

fn is_strict_snapshot_entry_order(entries: &[DependencySnapshotEntry]) -> bool {
    entries.windows(2).all(|pair| {
        pair[0]
            .compare_key(&pair[1])
            .then(pair[0].cached_version.cmp(&pair[1].cached_version))
            .is_lt()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::aspect::Aspect;

    #[test]
    fn stable_shape_basis_proves_against_previous_snapshot_shape() {
        let source = NodeId::new(2, 0);
        let mut snapshot = DependencySnapshot::empty();
        snapshot.record(source, Aspect::new(0), 5, None);
        let scan = DependencyInputScan::stable_shape(
            NodeId::new(0, 0),
            DependencySnapshotId::EMPTY,
            1,
            1,
            vec![8],
        );
        let mut store = super::super::DependencySnapshotShapeStore::default();
        let proof = StableShapeSnapshotBasis::prove(&scan, snapshot.shape().intern(&mut store))
            .expect("stable-shape scan should produce a proof");

        assert_eq!(proof.entry_count(), 1);
        assert_ne!(proof.shape_handle(), SnapshotShapeHandle::EMPTY);
    }
}
