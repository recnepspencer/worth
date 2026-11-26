//! The selected directory's lazy commit index and its typed refusals.
//!
//! A directory opens its paged commit index without reading a page; every
//! read is confined to the manifest's admitted page roster and protected by
//! the directory's own member basis, so it needs no custody of its own.
use std::sync::Arc;

use super::RelationalCheckpointDirectory;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::execution::denial_cause::RelationalExecutionDenialCause;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry as Entry;
use worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;

/// A commit-index refusal, attributed to the envelope position it serves.
pub fn unavailable(
    position: u64,
    cause: RelationalStorageUnavailableCause,
) -> RelationalStorageUnavailable {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Envelope(PatchStreamPosition(position)),
        cause,
    }
}

pub fn stop(position: u64, cause: RelationalStorageUnavailableCause) -> RelationalRecoveryStop {
    RelationalExecutionDenialCause::StorageUnavailable(unavailable(position, cause)).into()
}

impl RelationalCheckpointDirectory {
    pub fn commit_index(&self) -> &Arc<ColdCommitIndex> {
        &self.commit_index
    }

    pub fn commit_at(
        &self,
        position: PatchStreamPosition,
    ) -> Result<Option<Entry>, RelationalRecoveryStop> {
        self.commit_index
            .at_position(position)
            .map_err(|cause| stop(position.0, cause))
    }

    pub fn commit_entry(&self, commit: CommitId) -> Result<Option<Entry>, RelationalRecoveryStop> {
        self.commit_index
            .by_commit(commit)
            .map_err(|cause| stop(0, cause))
    }

    /// Visits every entry in position order, one page resident at a time.
    /// Only whole-history consumers (expanded decode, export) call this.
    pub fn for_each_commit<E: From<RelationalRecoveryStop>>(
        &self,
        mut visit: impl FnMut(&Entry) -> Result<(), E>,
    ) -> Result<(), E> {
        let pages = &self.commit_index.descriptor().pages;
        for (ordinal, reference) in pages.iter().enumerate() {
            let page = self
                .commit_index
                .page(ordinal)
                .map_err(|cause| E::from(stop(reference.first, cause)))?;
            for row in &page.page.rows {
                visit(row)?;
            }
        }
        Ok(())
    }
}
