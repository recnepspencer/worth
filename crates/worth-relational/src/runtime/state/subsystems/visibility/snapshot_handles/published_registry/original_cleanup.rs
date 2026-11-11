//! Exact original-cell removal, retaining actual custody before unlocking.
use super::super::published_release::PublishedSnapshotRetirement;
use super::super::PublishedSnapshotCapacityOwner;
use super::*;

impl PublishedSnapshotHandleRegistry {
    pub(crate) fn try_remove_original(
        &self,
        original: &Arc<PublishedSnapshotReleaseObservation>,
        capacity: &PublishedSnapshotCapacityOwner,
    ) -> Result<Option<bool>, ()> {
        use std::sync::TryLockError;
        let mut entries = match self.entries.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Ok(None),
        };
        let mut phase = match original.retirement.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Ok(None),
        };
        let snapshot_id = original.snapshot_id();
        match entries.by_id.get(&snapshot_id) {
            Some(entry) if !Arc::ptr_eq(&entry.release, original) => return Err(()),
            None if !matches!(*phase, PublishedSnapshotRetirement::Registered) => {
                return Ok(Some(false))
            }
            None => return Err(()),
            Some(_) => {}
        }
        if !matches!(*phase, PublishedSnapshotRetirement::Registered) {
            return Err(());
        }
        let registration = entries.remove(snapshot_id).expect(/* Proof: The exact entry and original release cell were checked under this same registry mutex immediately above. */ "checked published registration remains present");
        *phase = PublishedSnapshotRetirement::removed(registration.binding);
        original.mark_released();
        capacity.release();
        Ok(Some(true))
    }
}
