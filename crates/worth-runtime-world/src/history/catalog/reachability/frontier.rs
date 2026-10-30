//! Intrusive, deduplicated changed-slot frontier. Marking never allocates.
use super::{lock_slot, HistoryReachabilityIndex};
use crate::identity::CompositeCommitIdentity;

impl HistoryReachabilityIndex {
    pub(super) fn mark_changed(&mut self, identity: &CompositeCommitIdentity) {
        {
            let mut slot = lock_slot(self.records.get(identity).expect("installed changed slot"));
            let record = slot.as_mut().expect("installed changed record");
            if record.dirty {
                return;
            }
            record.dirty = true;
            record.dirty_previous = self.dirty_tail.clone();
            record.dirty_next = None;
        }
        if let Some(tail) = &self.dirty_tail {
            lock_slot(self.records.get(tail).expect("queued tail slot"))
                .as_mut()
                .expect("queued tail record")
                .dirty_next = Some(identity.clone());
        } else {
            self.dirty_head = Some(identity.clone());
        }
        self.dirty_tail = Some(identity.clone());
    }

    pub(in crate::history::catalog) fn pop_changed(&mut self) -> Option<CompositeCommitIdentity> {
        let identity = self.dirty_head.clone()?;
        self.forget_changed(&identity);
        Some(identity)
    }

    pub(super) fn forget_changed(&mut self, identity: &CompositeCommitIdentity) {
        let (previous, next) = {
            let mut slot = lock_slot(self.records.get(identity).expect("changed slot exists"));
            let record = slot.as_mut().expect("changed record installed");
            if !record.dirty {
                return;
            }
            record.dirty = false;
            (record.dirty_previous.take(), record.dirty_next.take())
        };
        if let Some(previous) = &previous {
            lock_slot(self.records.get(previous).expect("previous queued slot"))
                .as_mut()
                .expect("previous queued record")
                .dirty_next = next.clone();
        } else {
            self.dirty_head = next.clone();
        }
        if let Some(next) = &next {
            lock_slot(self.records.get(next).expect("next queued slot"))
                .as_mut()
                .expect("next queued record")
                .dirty_previous = previous.clone();
        } else {
            self.dirty_tail = previous;
        }
    }
}
