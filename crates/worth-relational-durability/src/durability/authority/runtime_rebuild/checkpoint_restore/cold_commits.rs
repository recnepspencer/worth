//! The selected checkpoint's commit index as the ledger's cold catalog base.
//!
//! Faults resolve the runtime's currently selected directory, falling back to
//! the reopened one until selection installs it; a later checkpoint covers
//! the same prefix, so faults follow it without rebinding cold commits. The
//! source holds no directory itself: close retires every directory through
//! its own edges, after which cold faults refuse as unavailable.
use std::sync::{Arc, Weak};

use crate::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit;
use crate::durability::data::RelationalCheckpointDirectory;
use crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection;
use branch::history::commit::artifact::RelationalCommitArtifact;
use branch::history::commit::cold_catalog::ColdCommitSource;
use branch::history::commit::index_entry::RelationalCommitIndexEntry;
use branch::history::commit::paged_index::reader::ColdCommitIndex;
use branch::history::commit::retained_index::RetainedCanonicalIndex;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::durability::medium::checkpoint_acknowledgement_denial::RelationalCheckpointAcknowledgementDenial as Ack;
use worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant;

#[derive(Debug)]
pub struct SelectedCommitSource {
    selection: RelationalCheckpointManifestSelection,
    reopened: Weak<RelationalCheckpointDirectory>,
    working_set: Option<Arc<WorkingSetAccountant>>,
}

impl SelectedCommitSource {
    pub fn new(
        selection: RelationalCheckpointManifestSelection,
        reopened: &Arc<RelationalCheckpointDirectory>,
        working_set: Option<Arc<WorkingSetAccountant>>,
    ) -> Self {
        Self {
            selection,
            reopened: Arc::downgrade(reopened),
            working_set,
        }
    }

    fn directory(&self) -> Option<Arc<RelationalCheckpointDirectory>> {
        self.selection
            .snapshot()
            .or_else(|| self.reopened.upgrade())
    }
}

impl ColdCommitSource for SelectedCommitSource {
    fn index(&self) -> Option<Arc<ColdCommitIndex>> {
        let index = self.directory()?.commit_index().clone();
        if let Some(accountant) = &self.working_set {
            index.bind_working_set(accountant.clone());
        }
        Some(index)
    }

    fn artifact(
        &self,
        entry: &RelationalCommitIndexEntry,
    ) -> Result<Arc<RelationalCommitArtifact>, Cause> {
        let directory = self.directory().ok_or(Cause::SourceUnavailable)?;
        let index = RetainedCanonicalIndex::copy_cold(entry, readmit_journal_commit);
        let artifact =
            RelationalCommitArtifact::from_index(index.clone(), directory.source.clone())
                .map_err(|_| Cause::Damaged)?;
        let _displaced = artifact
            .envelope_residency()
            .install_selected(index, directory)
            .map_err(|denial| match denial {
                Ack::Storage(cause) => cause,
                _ => Cause::SourceUnavailable,
            })?;
        Ok(Arc::new(artifact))
    }
}
