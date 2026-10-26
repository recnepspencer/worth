//! Head reconstruction uses the catalog's indexed envelope demand path and
//! preserves the canonical position admission issued by that readmission.
use super::RelationalCommitArtifact;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::data::PositionedCanonicalCommit;
use std::sync::Arc;

impl RelationalCommitArtifact {
    pub(crate) fn readmit_head(
        entry: crate::history::RelationalCommitIndexEntry,
        source: Arc<dyn RelationalDurableColdSource>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        let artifact = Self::from_index(entry, source, None)
            .map_err(|denial| corrupt(format!("head catalog identity: {denial:?}")))?;
        artifact.demand_envelope(execution, 1 << 30)?;
        let cell = Arc::try_unwrap(artifact.envelope)
            .map_err(|_| corrupt("head readmission retained an unexpected observer"))?;
        cell.into_positioned()
    }
}

fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
