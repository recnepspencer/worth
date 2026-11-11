//! Journal descriptions and streams through an actual selected read owner.
use super::{
    OrderedJournal, OrderedJournalEntry, OrderedJournalPayloadStream, OrderedJournalReadFailure,
};
use crate::physical_runtime::AnchoredRecordReadMembership;
use worth_execution::ExecutionRequest;

impl OrderedJournal<'_> {
    /// The journal head in this selected read's ORIGINAL reader; no new root
    /// is acquired and no raw reader or journal mutation authority escapes.
    pub fn head_position_from_read(
        &self,
        read: &AnchoredRecordReadMembership,
    ) -> Result<u64, OrderedJournalReadFailure> {
        self.head_position_from(read.original_reader())
    }
    /// Descriptive funded header metadata, not verified payload evidence.
    pub fn describe_entry_from_read(
        &self,
        read: &AnchoredRecordReadMembership,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
    ) -> Result<OrderedJournalEntry, OrderedJournalReadFailure> {
        self.describe_entry_from(read.original_reader(), request, position)
    }
    /// The same original-request-controlled payload algorithm and reader.
    pub fn open_payload_from_read<'request, 'authority>(
        &self,
        read: &AnchoredRecordReadMembership,
        request: &ExecutionRequest<'request, 'authority>,
        position: u64,
        limit: u64,
    ) -> Result<OrderedJournalPayloadStream<'request, 'authority>, OrderedJournalReadFailure> {
        self.open_payload_from(read.original_reader(), request, position, limit)
    }
}
