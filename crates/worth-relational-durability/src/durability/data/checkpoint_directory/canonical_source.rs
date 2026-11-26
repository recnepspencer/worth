//! Native directory verification behind a branch cell's selected-source port.
use worth_relational_branch::internal_port as branch;
use super::*;
use worth_relational_storage::internal_port::durability::medium::checkpoint_acknowledgement_denial::RelationalCheckpointAcknowledgementDenial as Ack;
use branch::history::commit::envelope_residency::selected_checkpoint::SelectedCanonicalDirectory;
use branch::history::commit::index_entry::RelationalCommitIndexEntry;
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_execution::ExecutionRequest;

impl SelectedCanonicalDirectory for RelationalCheckpointDirectory {
    fn source(&self) -> &Arc<dyn RelationalDurableColdSource> {
        &self.source
    }

    fn require_entry(&self, entry: &RelationalCommitIndexEntry) -> Result<(), Ack> {
        let row = self
            .manifest
            .canonical_members
            .member(entry.position)
            .ok_or(Ack::Storage(Cause::MissingMember))?;
        let indexed = self
            .commit_index
            .at_position(entry.position)
            .map_err(Ack::Storage)?;
        if indexed.as_ref() != Some(entry) || row.length < 66 {
            return Err(Ack::Storage(Cause::IntegrityMismatch));
        }
        Ok(())
    }

    fn readmit_canonical_member(
        &self,
        entry: &RelationalCommitIndexEntry,
        execution: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop>{
        Self::readmit_canonical_member(self, entry, execution, limit)
    }

    fn preflight_retirement(&self, source: &dyn RelationalDurableColdSource) -> Result<(), Ack> {
        Self::preflight_retirement(self, source)
    }
}
