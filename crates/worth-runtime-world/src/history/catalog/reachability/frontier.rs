//! Catalog row custody for the shared allocation-free dirty frontier.
use super::{lock_slot, HistoryReachabilityIndex};
use crate::identity::CompositeCommitIdentity;

impl HistoryReachabilityIndex {
    pub(in crate::history::catalog) fn mark_changed(&mut self, identity: &CompositeCommitIdentity) {
        let mut slot = lock_slot(self.records.get(identity).expect("installed changed slot"));
        let record = slot.as_mut().expect("installed changed record");
        self.dirty
            .mark(identity.clone(), &mut record.dirty, |neighbor, edit| {
                lock_slot(&self.records[&neighbor])
                    .as_mut()
                    .expect("installed dirty neighbor")
                    .dirty
                    .apply(edit);
            });
    }

    pub(in crate::history::catalog) fn pop_changed(&mut self) -> Option<CompositeCommitIdentity> {
        let identity = self.dirty.head()?;
        self.forget_changed(&identity);
        Some(identity)
    }

    pub(super) fn forget_changed(&mut self, identity: &CompositeCommitIdentity) {
        let mut slot = lock_slot(self.records.get(identity).expect("installed changed slot"));
        let record = slot.as_mut().expect("installed changed record");
        self.dirty.forget(&mut record.dirty, |neighbor, edit| {
            lock_slot(&self.records[&neighbor])
                .as_mut()
                .expect("installed dirty neighbor")
                .dirty
                .apply(edit);
        });
    }
}
