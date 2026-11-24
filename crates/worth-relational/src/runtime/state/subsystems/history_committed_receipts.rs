//! Immutable receipt selection across settled records and exact performed routes.
use super::history_canonical_routes::RelationalPublicationHistory;
use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

impl RelationalPublicationHistory {
    pub(crate) fn historical_latest_commit(&self) -> Option<RelationalCommitReceipt> {
        self.canonical_read().latest_receipt()
    }

    pub(crate) fn immutable_commit_receipt_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<RelationalCommitReceipt> {
        self.canonical_read().receipt_for_version(version_id)
    }

    pub(crate) fn latest_patch_stream_position(&self) -> Option<PatchStreamPosition> {
        self.canonical_read().latest_patch_stream_position()
    }
}
