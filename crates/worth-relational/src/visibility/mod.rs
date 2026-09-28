pub(crate) mod authority;
pub(crate) mod branch_scope;
pub(crate) mod cache_state;
pub(crate) mod exact_commit_snapshot;
pub(crate) mod materialization;
pub(crate) mod pins;
pub(crate) mod residency;
pub(crate) mod retention;
mod snapshot_admission;
mod snapshot_release;
pub(crate) mod snapshot_states;
pub(crate) mod store_correlation_reference;

pub use snapshot_admission::RelationalSnapshotAdmissionDenial;
pub use snapshot_release::{RelationalSnapshotReleaseDenial, RelationalSnapshotReleaseReceipt};
