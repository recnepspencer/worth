//! Dependency contracts, snapshot observations, and graph-owned storage.

mod edge;
mod snapshot_delta;
mod snapshot_observation;
mod snapshot_shape;
mod snapshot_storage;
mod snapshot_update;

pub use edge::{CanonicalDependencies, DependencyEdge, DependencySortKey};
pub use snapshot_delta::SnapshotDeltaRecord;
pub use snapshot_observation::{
    DependencyInputScan, DependencySnapshot, DependencySnapshotEntry, SnapshotChangeKind,
    StableShapeSnapshotBasis, VersionOnlySnapshotUpdate, VersionVector,
};
pub use snapshot_shape::{
    DependencySnapshotShape, DependencySnapshotShapeStore, SnapshotShapeHandle,
};
pub use snapshot_storage::{
    DependencySnapshotId, DependencySnapshotStore, SharedDependencySnapshot,
};
pub(crate) use snapshot_storage::{DependencySnapshotIndexDenial, PreparedSnapshotInsertion};
pub use snapshot_update::{
    CommittedSnapshotUpdate, ReplacementSnapshotUpdate, SnapshotStorageStrategy,
};
