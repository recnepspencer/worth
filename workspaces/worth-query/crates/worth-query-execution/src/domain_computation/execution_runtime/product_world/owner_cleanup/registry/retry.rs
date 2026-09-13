use std::sync::{Mutex, MutexGuard};

use super::{
    lock, WorthQueryProductBranchOwnerCleanupDenial, WorthQueryProductBranchOwnerCleanupRecord,
};

pub(super) struct RetryRecordGuard<'entry> {
    slot: MutexGuard<'entry, Option<WorthQueryProductBranchOwnerCleanupRecord>>,
    record: Option<WorthQueryProductBranchOwnerCleanupRecord>,
    armed: bool,
}

impl<'entry> RetryRecordGuard<'entry> {
    pub(super) fn take(
        slot: &'entry Mutex<Option<WorthQueryProductBranchOwnerCleanupRecord>>,
    ) -> Result<Self, WorthQueryProductBranchOwnerCleanupDenial> {
        let mut slot = lock(slot);
        let record = slot
            .take()
            .ok_or(WorthQueryProductBranchOwnerCleanupDenial::CleanupBusy)?;
        Ok(Self {
            slot,
            record: Some(record),
            armed: true,
        })
    }

    pub(super) fn record_mut(&mut self) -> &mut WorthQueryProductBranchOwnerCleanupRecord {
        self.record
            .as_mut()
            .expect("an armed retry guard owns the cleanup record")
    }

    pub(super) fn complete(mut self) {
        self.armed = false;
        drop(self.record.take());
    }
}

impl Drop for RetryRecordGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            *self.slot = self.record.take();
        }
    }
}
