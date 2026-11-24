//! Borrowed owner access across the sealed Relational package boundary.
use super::*;

#[doc(hidden)]
pub trait CommitResultOwnerPort: Sized {
    fn snapshot_release_observation(&self) -> &Arc<PublishedSnapshotReleaseObservation>;
}

impl CommitResultOwnerPort for CommitResult {
    fn snapshot_release_observation(&self) -> &Arc<PublishedSnapshotReleaseObservation> {
        self.snapshot_release_observation()
    }
}
