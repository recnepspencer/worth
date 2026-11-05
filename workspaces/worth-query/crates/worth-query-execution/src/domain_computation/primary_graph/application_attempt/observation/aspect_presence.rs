//! Borrow only presence at the exact decision/currentness observation.
use worth_foundational::facade::AspectKey;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{ProjectionAspectScope, RelationalRuntime},
    snapshots::SnapshotHandle,
    storage::RecordLifecycleState,
};

