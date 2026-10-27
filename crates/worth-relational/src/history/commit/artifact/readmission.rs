//! Head readmission preserves the canonical position proof from the journal.
use super::RelationalCommitArtifact;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::data::PositionedCanonicalCommit;
use std::sync::Arc;
impl RelationalCommitArtifact {
    pub(crate) fn readmit_head(
        entry: crate::history::RelationalCommitIndexEntry,
        source: Arc<dyn RelationalDurableColdSource>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        super::super::RelationalCommitEnvelopeResidency::readmit_positioned(
            &entry,
            source.as_ref(),
            execution,
            1 << 30,
        )
    }
}
