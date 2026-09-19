//! Work-admitted insertion into append-only snapshot storage.
use super::{
    DependencySnapshot, DependencySnapshotId, DependencySnapshotShapeStore,
    DependencySnapshotStore, SnapshotShapeHandle,
};
use crate::data::dependency::snapshot_shape::PreparedShapeInsertion;
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;

#[derive(Debug)]
pub(crate) struct PreparedSnapshotInsertion {
    pub(super) id: DependencySnapshotId,
    pub(super) action: InsertionAction,
}

#[derive(Debug)]
pub(super) enum InsertionAction {
    Existing(SnapshotShapeHandle),
    Append {
        snapshot: DependencySnapshot,
        shape: PreparedShapeInsertion,
        expected_len: usize,
    },
}

impl DependencySnapshotStore {
    pub(crate) fn prepare_insertion(
        &mut self,
        snapshot: DependencySnapshot,
        shapes: &mut DependencySnapshotShapeStore,
        work: &mut EvaluationWork<'_>,
    ) -> Result<PreparedSnapshotInsertion, SignalError> {
        let snapshot = snapshot.canonicalize_with_work(work)?;
        if snapshot.entries().is_empty() {
            return Ok(PreparedSnapshotInsertion {
                id: DependencySnapshotId::EMPTY,
                action: InsertionAction::Existing(SnapshotShapeHandle::EMPTY),
            });
        }
        match work {
            EvaluationWork::Ordinary => {
                self.rebuild_interner_if_needed();
                self.rebuild_shape_handles_if_needed(shapes);
            }
            EvaluationWork::Conditional(_) => self
                .require_retained_indexes(shapes)
                .map_err(|_| SignalError::SnapshotIndexUnavailable)?,
        }
        work.reserve(Some(snapshot.entries().len()))?;
        let mut bytes = Some(16usize);
        for entry in snapshot.entries() {
            bytes = bytes
                .and_then(|n| n.checked_add(32))
                .and_then(|n| {
                    n.checked_add(entry.scope.as_ref().map_or(0, |s| s.partition.0.len()))
                })
                .and_then(|n| {
                    n.checked_add(
                        entry
                            .scope
                            .as_ref()
                            .and_then(|s| s.detail.as_ref())
                            .map_or(0, String::len),
                    )
                });
        }
        let comparison = bytes.and_then(|n| n.checked_mul(2));
        let steps = self.interner.lookup_steps();
        work.reserve(comparison.and_then(|n| n.checked_mul(steps)))?;
        if let Some(id) = self.interner.get(&snapshot).copied() {
            work.reserve(Some(self.shape_handles.lookup_steps()))?;
            let handle = self
                .shape_handles
                .get(id.index().expect("stored snapshot id") - 1)
                .copied()
                .ok_or(SignalError::SnapshotIndexUnavailable)?;
            return Ok(PreparedSnapshotInsertion {
                id,
                action: InsertionAction::Existing(handle),
            });
        }
        // This closed owner never removes interner keys. A missing query cannot
        // enter retired-key readmission. Path copies clone Arc-backed snapshot
        // keys and fixed IDs, never another snapshot's scoped payload.
        work.reserve(
            comparison
                .and_then(|n| n.checked_mul(steps))
                .and_then(|n| n.checked_mul(3))
                .and_then(|n| n.checked_add(steps.checked_mul(16)?)),
        )?;
        work.reserve(append_work(&self.snapshots))?;
        work.reserve(append_work(&self.shape_handles))?;
        let next = self
            .snapshots
            .len()
            .checked_add(1)
            .filter(|n| *n <= u32::MAX as usize)
            .ok_or_else(|| SignalError::invalid_input("snapshot handle capacity exhausted"))?;
        let shape = shapes.prepare_intern_with_work(snapshot.shape_with_work(work)?, work)?;
        Ok(PreparedSnapshotInsertion {
            id: DependencySnapshotId::from_index(next),
            action: InsertionAction::Append {
                snapshot,
                shape,
                expected_len: self.snapshots.len(),
            },
        })
    }
}

impl PreparedSnapshotInsertion {
    pub(crate) fn snapshot_id(&self) -> DependencySnapshotId {
        self.id
    }

    pub(crate) fn publish(
        self,
        store: &mut DependencySnapshotStore,
        shapes: &mut DependencySnapshotShapeStore,
    ) -> (DependencySnapshotId, SnapshotShapeHandle) {
        let handle = match self.action {
            InsertionAction::Existing(handle) => handle,
            InsertionAction::Append {
                snapshot,
                shape,
                expected_len,
            } => {
                assert_eq!(
                    store.snapshots.len(),
                    expected_len,
                    "snapshot insertion must remain inside exclusive preparation/publication"
                );
                let handle = shape.publish(shapes);
                store.snapshots.push_back(snapshot.clone());
                store.shape_handles.push_back(handle);
                store.interner.insert(snapshot, self.id);
                handle
            }
        };
        (self.id, handle)
    }
}

// push_back relocates inline entries or copies fixed page Arc handles. It does
// not clone T payloads; snapshot cloning above is itself a fixed Arc increment.
fn append_work<T: Clone>(
    vector: &crate::data::persistent_vector::PersistentVector<T>,
) -> Option<usize> {
    match vector.exclusive_capacity() {
        Some(capacity) if capacity == vector.len() => vector
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(std::mem::size_of::<T>())),
        Some(_) => Some(1),
        None => vector
            .lookup_steps()
            .checked_mul(16)
            .and_then(|n| n.checked_add(32 * std::mem::size_of::<T>() + 64)),
    }
}

#[cfg(test)]
mod tests;
