use super::cursor::RetainedAuthoredWriteSelection;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};
use std::{num::NonZeroUsize, sync::Arc};

/// Bounds new page storage and actual relevant occurrence visits.
#[derive(Clone, Copy, Debug)]
pub struct RelationalAuthoredWritePageLimits {
    pub maximum_entries: NonZeroUsize,
    pub maximum_source_visits: NonZeroUsize,
    pub maximum_owned_bytes: NonZeroUsize,
}

/// A genuine canonical occurrence reacquired from its current Native history owner.
/// Its receipt is descriptive; this grants no Product, correction, or disclosure permission.
#[derive(Clone, Debug)]
pub struct RelationalAuthoredWriteOccurrence {
    pub(super) canonical: Arc<CanonicalCommitEnvelope>,
}
impl RelationalAuthoredWriteOccurrence {
    pub fn receipt(&self) -> &RelationalCommitReceipt {
        &self.canonical.commit
    }
    pub fn commit_id(&self) -> CommitId {
        self.canonical.commit.commit_id
    }
    /// Borrow the actual full-generation allocations co-committed by this
    /// canonical occurrence. Admit the roster length before traversing it.
    pub fn allocated_records(
        &self,
    ) -> impl ExactSizeIterator<Item = &crate::transactions::data::RecordRef> {
        self.canonical
            .record_allocations()
            .iter()
            .map(|allocation| allocation.record())
    }
}

#[derive(Debug)]
pub struct RelationalAuthoredWritePage {
    pub(super) slots: worth_execution::ExecutionArray<Option<RelationalAuthoredWriteOccurrence>>,
    pub(super) count: usize,
    pub(super) next: u64,
    pub(super) complete: bool,
    pub(super) visits: usize,
}
impl RelationalAuthoredWritePage {
    pub fn entries(&self) -> impl ExactSizeIterator<Item = &RelationalAuthoredWriteOccurrence> {
        self.slots[..self.count]
            .iter()
            .map(|slot| slot.as_ref().expect("populated prefix"))
    }
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
    pub const fn source_visits(&self) -> usize {
        self.visits
    }
}

/// Dropping staged work leaves the cursor's accepted position unchanged.
#[derive(Debug)]
pub struct RelationalPreparedAuthoredWritePage<'request, 'authority> {
    pub(super) source: Arc<RetainedAuthoredWriteSelection>,
    pub(super) expected: u64,
    pub(super) page: RelationalAuthoredWritePage,
    pub(super) execution: worth_execution::ExecutionRequest<'request, 'authority>,
}
impl RelationalPreparedAuthoredWritePage<'_, '_> {
    pub fn page(&self) -> &RelationalAuthoredWritePage {
        &self.page
    }
}
