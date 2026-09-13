use super::{ProductUnpublishedOwnerEffectsIdentity, ProductUnpublishedOwnerEffectsRecord};
use crate::publication::ActiveAttemptRecord;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub(super) enum RecoveryEntry {
    Active(Arc<ActiveAttemptRecord>),
    Retained(Arc<ProductUnpublishedOwnerEffectsRecord>),
    Busy,
}

#[derive(Debug)]
enum RecoverySlot {
    Vacant {
        next: Option<usize>,
    },
    Occupied {
        identity: ProductUnpublishedOwnerEffectsIdentity,
        entry: RecoveryEntry,
    },
}

/// One slot per admitted attempt, reused without growing with lifetime churn.
/// Only admission inserts/grows indexes. Terminal and recovery transitions
/// replace the resident value without allocating another map entry.
#[derive(Debug, Default)]
pub(super) struct RecoveryRecordSlots {
    slots: Vec<RecoverySlot>,
    free: Option<usize>,
    index: HashMap<ProductUnpublishedOwnerEffectsIdentity, usize>,
    retained: usize,
}

impl RecoveryRecordSlots {
    pub(super) const fn metadata_charge_hint() -> usize {
        std::mem::size_of::<RecoverySlot>()
            + std::mem::size_of::<ProductUnpublishedOwnerEffectsIdentity>()
            + std::mem::size_of::<usize>()
    }
    pub(super) fn retained_len(&self) -> usize {
        self.retained
    }
    pub(super) fn span(&self) -> usize {
        self.slots.len()
    }
    pub(super) fn at(
        &self,
        position: usize,
    ) -> Option<(&ProductUnpublishedOwnerEffectsIdentity, &RecoveryEntry)> {
        match self.slots.get(position)? {
            RecoverySlot::Occupied { identity, entry } => Some((identity, entry)),
            RecoverySlot::Vacant { .. } => None,
        }
    }
    pub(super) fn get(
        &self,
        identity: &ProductUnpublishedOwnerEffectsIdentity,
    ) -> Option<&RecoveryEntry> {
        self.at(*self.index.get(identity)?).map(|(_, entry)| entry)
    }
    pub(super) fn insert_active(&mut self, record: Arc<ActiveAttemptRecord>) {
        let identity = record.identity().clone();
        assert!(!self.index.contains_key(&identity));
        let occupied = RecoverySlot::Occupied {
            identity: identity.clone(),
            entry: RecoveryEntry::Active(record),
        };
        let position = if let Some(position) = self.free {
            let RecoverySlot::Vacant { next } = self.slots[position] else {
                unreachable!("free slot")
            };
            self.free = next;
            self.slots[position] = occupied;
            position
        } else {
            let position = self.slots.len();
            self.slots.push(occupied);
            position
        };
        assert!(self.index.insert(identity, position).is_none());
    }
    pub(super) fn replace(
        &mut self,
        identity: &ProductUnpublishedOwnerEffectsIdentity,
        next: RecoveryEntry,
    ) -> RecoveryEntry {
        let position = self.index[identity];
        let RecoverySlot::Occupied { entry, .. } = &mut self.slots[position] else {
            unreachable!("indexed slot")
        };
        self.retained += usize::from(matches!(next, RecoveryEntry::Retained(_)));
        let old = std::mem::replace(entry, next);
        self.retained -= usize::from(matches!(old, RecoveryEntry::Retained(_)));
        old
    }
    pub(super) fn remove(
        &mut self,
        identity: &ProductUnpublishedOwnerEffectsIdentity,
    ) -> Option<RecoveryEntry> {
        let position = self.index.remove(identity)?;
        let removed = std::mem::replace(
            &mut self.slots[position],
            RecoverySlot::Vacant { next: self.free },
        );
        self.free = Some(position);
        let RecoverySlot::Occupied { entry, .. } = removed else {
            unreachable!("indexed slot")
        };
        self.retained -= usize::from(matches!(entry, RecoveryEntry::Retained(_)));
        Some(entry)
    }
    pub(super) fn iter(
        &self,
    ) -> impl Iterator<Item = (&ProductUnpublishedOwnerEffectsIdentity, &RecoveryEntry)> {
        (0..self.slots.len()).filter_map(|position| self.at(position))
    }
}
