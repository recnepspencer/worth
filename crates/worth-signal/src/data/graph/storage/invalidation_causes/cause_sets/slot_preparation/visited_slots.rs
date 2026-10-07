//! Duplicate tracking for one publication, with no heap for a single owner.
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct VisitedCauseSlots {
    first: Option<u32>,
    additional: BTreeSet<u32>,
}

impl VisitedCauseSlots {
    pub(super) fn len(&self) -> usize {
        usize::from(self.first.is_some()) + self.additional.len()
    }

    pub(super) fn insert(&mut self, slot: u32) -> bool {
        match self.first {
            None => {
                self.first = Some(slot);
                true
            }
            Some(first) if first == slot => false,
            Some(_) => self.additional.insert(slot),
        }
    }

    pub(super) fn contains(&self, slot: u32) -> bool {
        self.first == Some(slot) || self.additional.contains(&slot)
    }

    pub(super) fn remove(&mut self, slot: u32) -> bool {
        if self.first == Some(slot) {
            self.first = self.additional.pop_first();
            true
        } else {
            self.additional.remove(&slot)
        }
    }
}
