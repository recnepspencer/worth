//! Prepared append-only shape insertion; consumed within exclusive storage use.
use super::{DependencySnapshotShape, DependencySnapshotShapeStore, SnapshotShapeHandle};

#[derive(Debug)]
pub(in crate::data::dependency) struct PreparedShapeInsertion {
    pub(super) handle: SnapshotShapeHandle,
    pub(super) append: Option<(DependencySnapshotShape, usize)>,
}

impl PreparedShapeInsertion {
    pub(in crate::data::dependency) fn handle(&self) -> SnapshotShapeHandle {
        self.handle
    }

    pub(super) fn existing(handle: SnapshotShapeHandle) -> Self {
        Self {
            handle,
            append: None,
        }
    }
    pub(super) fn new(
        shape: DependencySnapshotShape,
        handle: SnapshotShapeHandle,
        expected_len: usize,
    ) -> Self {
        Self {
            handle,
            append: Some((shape, expected_len)),
        }
    }
    pub(in crate::data::dependency) fn publish(
        self,
        store: &mut DependencySnapshotShapeStore,
    ) -> SnapshotShapeHandle {
        if let Some((shape, expected_len)) = self.append {
            assert_eq!(
                store.shapes.len(),
                expected_len,
                "shape insertion must remain inside exclusive preparation/publication"
            );
            store.shapes.push_back(shape.clone());
            store.interner.insert(shape, self.handle);
        }
        self.handle
    }
}
