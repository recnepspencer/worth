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

impl super::RelationalCommitArtifact {
    /// Compare a recovered body against the retained commitment without loading it.
    pub(crate) fn matches_envelope(
        &self,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<bool, super::RelationalCommitArtifactDenial> {
        if let Some(resident) = self.resident_envelope() {
            return Ok(resident.as_ref() == envelope);
        }
        let entry = self
            .envelope_residency()
            .journal_index()
            .ok_or(super::RelationalCommitArtifactDenial::RootLinkage)?;
        let digest = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::envelope_payload_digest(entry.position, envelope)
            .map_err(|_| super::RelationalCommitArtifactDenial::CanonicalPayloadEncoding)?;
        Ok(entry.commit == envelope.commit
            && entry.schema_version == envelope.schema_version
            && entry.payload_digest == digest)
    }
}
