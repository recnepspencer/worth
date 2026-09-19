use super::{DependencySnapshotShapeStore, DependencySnapshotStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencySnapshotIndexDenial {
    SnapshotInternerRequiresReconstruction,
    ShapeHandlesRequireReconstruction,
    ShapeInternerRequiresReconstruction,
    SnapshotHandleUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::dependency::{DependencySnapshot, DependencySnapshotEntry};
    use crate::data::{aspect::Aspect, handle::NodeId};

    #[test]
    fn retained_snapshot_readiness_distinguishes_each_missing_index_without_repair() {
        let snapshot = DependencySnapshot::from_ordered_unique([DependencySnapshotEntry {
            source: NodeId::new(0, 0),
            aspect: Aspect::new(0),
            cached_version: 7,
            scope: None,
        }]);
        let mut snapshots = DependencySnapshotStore::default();
        let mut shapes = DependencySnapshotShapeStore::default();
        assert_eq!(snapshots.require_retained_indexes(&shapes), Ok(()));
        snapshots.insert(snapshot.clone());
        assert_eq!(
            snapshots.require_retained_indexes(&shapes),
            Err(DependencySnapshotIndexDenial::ShapeHandlesRequireReconstruction)
        );
        let (id, _) = snapshots.insert_with_shape_handle(snapshot, &mut shapes);
        assert_eq!(snapshots.require_retained_indexes(&shapes), Ok(()));
        let encoded = serde_json::to_string(&snapshots).unwrap();
        let restored: DependencySnapshotStore = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            restored.require_retained_indexes(&shapes),
            Err(DependencySnapshotIndexDenial::SnapshotInternerRequiresReconstruction)
        );
        assert_eq!(restored.get(id), snapshots.get(id));
        assert!(restored.interner.is_empty());
        let encoded_shapes = serde_json::to_string(&shapes).unwrap();
        let restored_shapes: DependencySnapshotShapeStore =
            serde_json::from_str(&encoded_shapes).unwrap();
        assert_eq!(
            snapshots.require_retained_indexes(&restored_shapes),
            Err(DependencySnapshotIndexDenial::ShapeInternerRequiresReconstruction)
        );
        assert_eq!(snapshots.require_retained_indexes(&shapes), Ok(()));
    }
}

impl DependencySnapshotStore {
    /// Retained execution must not enter lazy whole-store reconstruction.
    /// These cardinalities are carried by the storage owners; no scan or repair
    /// occurs here. Reconstitution remains an explicit, separately bounded lane.
    pub(crate) fn require_retained_indexes(
        &self,
        shapes: &DependencySnapshotShapeStore,
    ) -> Result<(), DependencySnapshotIndexDenial> {
        if self.interner.len() != self.snapshots.len() {
            return Err(DependencySnapshotIndexDenial::SnapshotInternerRequiresReconstruction);
        }
        if self.shape_handles.len() != self.snapshots.len() {
            return Err(DependencySnapshotIndexDenial::ShapeHandlesRequireReconstruction);
        }
        if !shapes.retained_interner_is_complete() {
            return Err(DependencySnapshotIndexDenial::ShapeInternerRequiresReconstruction);
        }
        Ok(())
    }
}

impl DependencySnapshotStore {
    /// This read never reconstructs either interner or the shape-handle vector.
    pub(crate) fn retained_shape_handle_for(
        &self,
        id: super::DependencySnapshotId,
        shapes: &DependencySnapshotShapeStore,
    ) -> Result<super::SnapshotShapeHandle, DependencySnapshotIndexDenial> {
        self.require_retained_indexes(shapes)?;
        let Some(index) = id.index() else {
            return Ok(super::SnapshotShapeHandle::EMPTY);
        };
        self.shape_handles
            .get(index - 1)
            .copied()
            .ok_or(DependencySnapshotIndexDenial::SnapshotHandleUnavailable)
    }

    pub(crate) fn retained_shape_handle_lookup_steps(&self) -> usize {
        self.shape_handles.lookup_steps() + 8
    }
}
