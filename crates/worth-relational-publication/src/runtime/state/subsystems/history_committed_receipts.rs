//! Immutable receipt selection across settled records and exact performed routes.
use super::history_canonical_routes::RelationalPublicationHistory;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::identity::data::primitives::VersionId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_relational_contracts::internal_port as contracts;

impl RelationalPublicationHistory {
    pub fn historical_latest_commit(&self) -> Option<RelationalCommitReceipt> {
        self.canonical_read().latest_receipt()
    }

    pub fn immutable_commit_receipt_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<RelationalCommitReceipt> {
        self.canonical_read().receipt_for_version(version_id)
    }

    pub fn latest_patch_stream_position(&self) -> Option<PatchStreamPosition> {
        self.canonical_read().latest_patch_stream_position()
    }
}
