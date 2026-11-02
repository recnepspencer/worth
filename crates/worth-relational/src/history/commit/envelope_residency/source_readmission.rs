//! Journal and selected singleton bytes share the complete existing admission.
use super::*;
impl RelationalCommitEnvelopeResidency {
    pub(crate) fn readmit_positioned(
        entry: &RelationalCommitIndexEntry,
        source: &dyn RelationalDurableColdSource,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        let bytes = source.read_commit(entry.position, limit).map_err(failure)?;
        let artifact = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes)
            .map_err(|_| failure(Cause::Damaged))?;
        Self::check_frame(entry, &artifact)?;
        let admitted = RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution)
            .map_err(|error| match error {
                crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(cause) => RelationalRecoveryStop::Request(cause),
                crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::FieldText(cause) => crate::symbols::data::field_text_recovery_stop(cause),
                _ => failure(Cause::Damaged).into(),
            })?.into_positioned();
        Self::check_positioned(entry, admitted, execution)
    }
    pub(super) fn readmit_owned_artifact(
        entry: &RelationalCommitIndexEntry,
        artifact: RelationalCanonicalCommitArtifact,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        Self::check_frame(entry, &artifact)?;
        let admitted=artifact.readmit_owned(execution).map_err(|error| match error {
            crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(cause)=>RelationalRecoveryStop::Request(cause),
            crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::FieldText(cause)=>crate::symbols::data::field_text_recovery_stop(cause),
            _=>RelationalStorageUnavailable{subject:RelationalStorageSubject::Envelope(entry.position),cause:Cause::Damaged}.into(),
        })?.into_positioned();
        Self::check_positioned(entry, admitted, execution)
    }
    pub(super) fn check_frame(
        entry: &RelationalCommitIndexEntry,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> Result<(), RelationalRecoveryStop> {
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        if artifact.position() != entry.position
            || artifact.commit_id() != entry.commit.commit_id
            || artifact.payload_digest().as_bytes() != &entry.payload_digest
        {
            return Err(failure(Cause::IntegrityMismatch).into());
        }
        Ok(())
    }
    fn check_positioned(
        entry: &RelationalCommitIndexEntry,
        admitted: PositionedCanonicalCommit,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        if admitted.commit != entry.commit
            || admitted.envelope().association_discovery() != entry.association_discovery
            || admitted.schema_version != entry.schema_version
            || entry.root.schema_root()
                != &crate::schema::data::schema_authority_snapshot_digest_bytes(
                    &admitted.schema_authority,
                )
        {
            return Err(failure(Cause::IntegrityMismatch).into());
        }
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        Ok(admitted)
    }
}
