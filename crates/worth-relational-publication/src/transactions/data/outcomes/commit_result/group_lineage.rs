//! The canonical group lineage a commit published.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_contracts::internal_port::aggregate_identity::transition::group::transition_result::AggregateGroupTransition;
use worth_relational_lineage::internal_port::durability::naming::{
    decode_group_lineage_frame, PersistentNameDurabilityError,
};

use super::CommitResult;

impl CommitResult {
    /// The group transitions this commit published, by installed group
    /// scheme, decoded from its canonical group lineage frame; empty when it
    /// touched no group.
    pub fn group_lineage(
        &self,
    ) -> Result<Vec<(String, AggregateGroupTransition)>, PersistentNameDurabilityError> {
        let decoded = decode_group_lineage_frame(self.group_lineage_frame())?;
        Ok(decoded
            .into_iter()
            .map(|committed| (committed.scheme_id().to_owned(), committed.transition))
            .collect())
    }

    /// The canonical WPGL group lineage frame bytes this commit published:
    /// `O(touched)` for a kept group's member edit, empty when it touched
    /// no group.
    pub fn group_lineage_frame(&self) -> &[u8] {
        self.envelope().published_lineage().group_lineage()
    }
}
