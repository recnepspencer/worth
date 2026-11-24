//! Sealed partition-change observations from the Storage journal.
use crate::identity::data::{KindId, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionMutationJournal;
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordChangeState {
    pub record: RecordRef,
    pub kind: KindId,
    pub lifecycle: RecordLifecycleState,
    pub retired_at: Option<VersionId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedRecord {
    pub before: Option<RecordChangeState>,
    pub after: Option<RecordChangeState>,
}

/// Sealed storage-owner journal; access never enumerates the selected partition.
#[derive(Debug)]
pub struct PartitionChange {
    pub(crate) journal: PartitionMutationJournal,
    pub(crate) records: Vec<ChangedRecord>,
}
impl PartitionChange {
    pub fn records(&self) -> &[ChangedRecord] {
        &self.records
    }
    pub fn entity_slots(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.journal.entity_slots.iter().copied()
    }
    pub fn relation_slots(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.journal.relation_slots.iter().copied()
    }
    pub fn outgoing_slots(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.journal.adjacency_slots.iter().copied()
    }
    pub fn incoming_slots(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.journal.reverse_adjacency_slots.iter().copied()
    }
}

impl ChangedRecord {
    pub(crate) fn created_identity(&self) -> Option<&RecordRef> {
        let after = self.after.as_ref()?;
        self.before
            .as_ref()
            .is_none_or(|before| before.record != after.record)
            .then_some(&after.record)
    }
    pub(crate) fn retired_identity(&self) -> Option<&RecordRef> {
        let before = self.before.as_ref()?;
        self.after
            .as_ref()
            .is_none_or(|after| {
                after.record != before.record
                    || after.retired_at != before.retired_at && after.retired_at.is_some()
            })
            .then_some(&before.record)
    }
}
