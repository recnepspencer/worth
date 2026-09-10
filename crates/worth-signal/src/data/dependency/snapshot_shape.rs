mod fork_growth;
mod insertion;
pub(super) use insertion::PreparedShapeInsertion;
mod reserved_fork;
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

mod interning;
mod retained_charge;
mod retained_publication;

use super::DependencySortKey;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct DependencySnapshotShape {
    keys: std::sync::Arc<Vec<DependencySortKey>>,
}

impl DependencySnapshotShape {
    pub fn from_ordered_unique(keys: impl IntoIterator<Item = DependencySortKey>) -> Self {
        let keys = keys.into_iter().collect::<Vec<_>>();
        debug_assert!(is_strict_snapshot_shape_order(keys.as_slice()));
        Self {
            keys: std::sync::Arc::new(keys),
        }
    }

    pub fn as_slice(&self) -> &[DependencySortKey] {
        self.keys.as_slice()
    }

    pub(crate) fn intern(&self, store: &mut DependencySnapshotShapeStore) -> SnapshotShapeHandle {
        store.intern(self.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SnapshotShapeHandle(Option<NonZeroU32>);

impl SnapshotShapeHandle {
    pub const EMPTY: Self = Self(None);

    fn from_index(index: usize) -> Self {
        debug_assert!(index > 0);
        Self(NonZeroU32::new(index as u32))
    }

    fn index(self) -> Option<usize> {
        self.0.map(|index| index.get() as usize)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DependencySnapshotShapeStore {
    shapes: crate::data::persistent_vector::PersistentVector<DependencySnapshotShape>,
    #[serde(skip, default)]
    interner: crate::data::persistent_ord_map::PersistentOrdMap<
        DependencySnapshotShape,
        SnapshotShapeHandle,
    >,
}

impl DependencySnapshotShapeStore {
    pub(super) fn retained_interner_is_complete(&self) -> bool {
        self.interner.len() == self.shapes.len()
    }

    fn rebuild_interner_if_needed(&mut self) {
        if !self.interner.is_empty() || self.shapes.is_empty() {
            return;
        }
        for (index, shape) in self.shapes.iter().cloned().enumerate() {
            self.interner
                .insert(shape, SnapshotShapeHandle::from_index(index + 1));
        }
    }

    pub fn intern(&mut self, shape: DependencySnapshotShape) -> SnapshotShapeHandle {
        self.intern_with_work(
            shape,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .expect("ordinary shape interning must remain representable")
    }

    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            shapes: self.shapes.operational_clone(),
            interner: self.interner.operational_clone(),
        }
    }

    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            shapes: self.shapes.fork_persistent(),
            interner: self.interner.fork_persistent(),
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            shapes: self.shapes.clone(),
            interner: self.interner.fork_storage_identity(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.shapes.shares_storage_with(&other.shapes) && self.interner.ptr_eq(&other.interner)
    }
}

fn is_strict_snapshot_shape_order(keys: &[DependencySortKey]) -> bool {
    keys.windows(2).all(|pair| pair[0] < pair[1])
}
