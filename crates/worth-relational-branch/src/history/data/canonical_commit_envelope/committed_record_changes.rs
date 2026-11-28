//! Selected record patches carry their exact canonical commit receipt.
use super::{CanonicalCommitEnvelopeBody, RelationalCommitReceipt};
use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommittedRecordChange<'a> {
    pub commit: &'a RelationalCommitReceipt,
    pub record: &'a PublishedAuthoritativeRecordPatch,
}

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn committed_record_changes(
        &self,
    ) -> impl Iterator<Item = CommittedRecordChange<'_>> {
        self.patch
            .authoritative_record_patches
            .iter()
            .map(|record| CommittedRecordChange {
                commit: &self.commit,
                record,
            })
    }

    pub(crate) fn committed_record_changes_for_target<'a>(
        &'a self,
        target: &'a RecordRef,
    ) -> impl Iterator<Item = CommittedRecordChange<'a>> + 'a {
        self.committed_record_changes()
            .filter(move |change| &change.record.target == target)
    }
}
