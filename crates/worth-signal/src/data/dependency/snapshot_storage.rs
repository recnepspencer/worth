mod fork_growth;
mod insertion;
pub(crate) use insertion::PreparedSnapshotInsertion;
mod reserved_fork;
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

mod retained_charge;
mod retained_publication;
mod retention_readiness;
pub(crate) use retention_readiness::DependencySnapshotIndexDenial;

use super::{DependencySnapshot, DependencySnapshotShapeStore, SnapshotShapeHandle};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedDependencySnapshot {
    snapshot: DependencySnapshot,
}

impl SharedDependencySnapshot {
    pub fn new(snapshot: DependencySnapshot) -> Self {
        Self { snapshot }
    }

    pub fn empty() -> Self {
        Self::new(DependencySnapshot::empty())
    }

    pub fn snapshot(&self) -> &DependencySnapshot {
        &self.snapshot
    }

    pub fn entries(&self) -> &[super::DependencySnapshotEntry] {
        self.snapshot.entries()
    }

    pub fn into_snapshot(self) -> DependencySnapshot {
        self.snapshot
    }

    /// Whether two shared snapshots currently point at the same backing
    /// storage. This is a storage-strategy fact, not semantic identity.
    pub fn shares_storage_with(&self, other: &Self) -> bool {
        self.snapshot.shares_storage_with(&other.snapshot)
    }
}

/// Stable handle into graph-owned dependency snapshot storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct DependencySnapshotId(Option<NonZeroU32>);

impl DependencySnapshotId {
    /// The canonical empty snapshot id.
    pub const EMPTY: Self = Self(None);

    fn from_index(index: usize) -> Self {
        debug_assert!(index > 0);
        Self(NonZeroU32::new(index as u32))
    }

    fn index(self) -> Option<usize> {
        self.0.map(|index| index.get() as usize)
    }

    pub(crate) fn from_semantic_fingerprint(fingerprint: u32) -> Self {
        match NonZeroU32::new(fingerprint) {
            Some(non_zero) => Self(Some(non_zero)),
            None => Self(Some(NonZeroU32::new(1).expect("1 is non-zero"))),
        }
    }
}

/// Graph-owned storage for immutable dependency snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DependencySnapshotStore {
    snapshots: crate::data::persistent_vector::PersistentVector<DependencySnapshot>,
    #[serde(skip, default)]
    interner:
        crate::data::persistent_ord_map::PersistentOrdMap<DependencySnapshot, DependencySnapshotId>,
    #[serde(skip, default)]
    shape_handles: crate::data::persistent_vector::PersistentVector<SnapshotShapeHandle>,
}

impl DependencySnapshotStore {
    fn rebuild_interner_if_needed(&mut self) {
        if !self.interner.is_empty() || self.snapshots.is_empty() {
            return;
        }
        for (index, snapshot) in self.snapshots.iter().cloned().enumerate() {
            self.interner
                .insert(snapshot, DependencySnapshotId::from_index(index + 1));
        }
    }

    pub(crate) fn lookup_steps(&self) -> usize {
        self.snapshots.lookup_steps()
            + crate::data::retained_storage::ordered_lookup_steps(
                self.snapshots.len().saturating_add(1),
            )
    }

    /// Read one snapshot by id.
    pub fn get(&self, id: DependencySnapshotId) -> &DependencySnapshot {
        match id.index() {
            Some(index) => &self.snapshots[index - 1],
            None => empty_dependency_snapshot(),
        }
    }

    /// Store one immutable snapshot and return its id.
    pub fn insert(&mut self, snapshot: DependencySnapshot) -> DependencySnapshotId {
        let snapshot = snapshot.canonicalize_unordered();
        if snapshot.entries().is_empty() {
            return DependencySnapshotId::EMPTY;
        }
        self.rebuild_interner_if_needed();
        if let Some(id) = self.interner.get(&snapshot).copied() {
            return id;
        }
        self.snapshots.push_back(snapshot);
        let id = DependencySnapshotId::from_index(self.snapshots.len());
        let snapshot = self.snapshots[id.index().expect("snapshot id should index") - 1].clone();
        self.interner.insert(snapshot, id);
        id
    }

    fn rebuild_shape_handles_if_needed(&mut self, shape_store: &mut DependencySnapshotShapeStore) {
        if self.shape_handles.len() == self.snapshots.len() {
            return;
        }
        self.shape_handles.clear();
        for snapshot in &self.snapshots {
            self.shape_handles
                .push_back(snapshot.shape().intern(shape_store));
        }
    }

    pub fn shape_handle_for(
        &mut self,
        id: DependencySnapshotId,
        shape_store: &mut DependencySnapshotShapeStore,
    ) -> SnapshotShapeHandle {
        let Some(index) = id.index() else {
            return SnapshotShapeHandle::EMPTY;
        };
        self.rebuild_shape_handles_if_needed(shape_store);
        self.shape_handles
            .get(index - 1)
            .copied()
            .unwrap_or(SnapshotShapeHandle::EMPTY)
    }

    pub fn insert_with_shape_handle(
        &mut self,
        snapshot: DependencySnapshot,
        shape_store: &mut DependencySnapshotShapeStore,
    ) -> (DependencySnapshotId, SnapshotShapeHandle) {
        let insertion = self
            .prepare_insertion(
                snapshot,
                shape_store,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary,
            )
            .expect("ordinary snapshot insertion must remain representable");
        insertion.publish(self, shape_store)
    }

    #[cfg(test)]
    pub(crate) fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub(crate) fn live_snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            snapshots: self.snapshots.operational_clone(),
            interner: self.interner.operational_clone(),
            shape_handles: self.shape_handles.operational_clone(),
        }
    }

    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            snapshots: self.snapshots.fork_persistent(),
            interner: self.interner.fork_persistent(),
            shape_handles: self.shape_handles.fork_persistent(),
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            snapshots: self.snapshots.clone(),
            interner: self.interner.fork_storage_identity(),
            shape_handles: self.shape_handles.clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.snapshots.shares_storage_with(&other.snapshots)
            && self.shape_handles.shares_storage_with(&other.shape_handles)
            && self.interner.ptr_eq(&other.interner)
    }
}

fn empty_dependency_snapshot() -> &'static DependencySnapshot {
    static EMPTY: std::sync::OnceLock<DependencySnapshot> = std::sync::OnceLock::new();
    EMPTY.get_or_init(DependencySnapshot::empty)
}
