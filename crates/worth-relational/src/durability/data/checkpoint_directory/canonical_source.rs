//! Native directory verification behind a branch cell's selected-source port.
use super::*;
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
use crate::history::commit::envelope_residency::selected_checkpoint::SelectedCanonicalDirectory;
use crate::history::commit::RelationalCommitIndexEntry;
use crate::history::data::PositionedCanonicalCommit;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
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
        let ordinal = usize::try_from(
            entry
                .position
                .0
                .checked_sub(1)
                .ok_or(Ack::Storage(Cause::IntegrityMismatch))?,
        )
        .map_err(|_| Ack::Storage(Cause::IntegrityMismatch))?;
        if self.manifest.commits.get(ordinal) != Some(entry) || row.length < 66 {
            return Err(Ack::Storage(Cause::IntegrityMismatch));
        }
        Ok(())
    }

    fn readmit_canonical_member(
        &self,
        entry: &RelationalCommitIndexEntry,
        execution: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, crate::durability::data::RelationalRecoveryStop> {
        Self::readmit_canonical_member(self, entry, execution, limit)
    }

    fn preflight_retirement(&self, source: &dyn RelationalDurableColdSource) -> Result<(), Ack> {
        Self::preflight_retirement(self, source)
    }
}
