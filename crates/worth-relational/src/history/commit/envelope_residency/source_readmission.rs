//! Journal and selected singleton bytes share the complete existing admission.
use super::*;
impl RelationalCommitEnvelopeResidency {
    /// Source admission validates original bytes even when the logical body is warm.
    pub(crate) fn readmit_original_positioned(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        {
            let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            if let Some(failure) = state.damage {
                return Err(failure.into());
            }
        }
        let (entry, _) = self
            .source_placement()
            .ok_or_else(|| RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(
                    self.captured_position()
                        .unwrap_or(crate::publication::patch::data::PatchStreamPosition(0)),
                ),
                cause: Cause::SourceUnavailable,
            })?;
        // Only Journal needs a new prefix read. Selected uses its retained cut.
        let journal = self.journal_placement().map(|(_, source)| source);
        let protection = journal
            .as_ref()
            .map(|source| {
                crate::durability::medium::RelationalDurableReadProtection::capture(
                    source, execution,
                )
                .map_err(|cause| {
                    crate::durability::checkpoint_history::capture::owned_stop(
                        cause,
                        RelationalStorageSubject::Envelope(entry.position),
                    )
                })
            })
            .transpose()?;
        let artifact =
            self.require_original_artifact(execution, journal.as_ref().zip(protection.as_ref()))?;
        let result = Self::readmit_owned_artifact(&entry, artifact, execution);
        if let Err(stop) = &result {
            self.state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remember_original_failure(stop);
        }
        result
    }
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
        let read_limit =
            limit.min(crate::durability::canonical_commit_artifact::MAXIMUM_ARTIFACT_BYTES as u64);
        let reservation = execution
            .reserve_retained_memory(read_limit)
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        let bytes = source
            .read_commit(entry.position, read_limit)
            .map_err(failure)?;
        let artifact = RelationalCanonicalCommitArtifact::copy_frame_original(&bytes, execution)
            .map_err(|cause| match cause {
                crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial::Request(cause) => RelationalRecoveryStop::Request(cause),
                crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial::Allocation(cause) => RelationalRecoveryStop::Allocation(cause),
                crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial::Frame(cause) => crate::durability::data::RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause).into(),
            })?;
        drop(bytes);
        drop(reservation);
        Self::check_frame(entry, &artifact)?;
        let admitted = artifact.readmit_framed(execution)
            .map_err(|error| match error {
                crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(cause) => RelationalRecoveryStop::Request(cause),
                crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Content(cause) => cause.into(),
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
        let admitted=artifact.readmit_framed(execution).map_err(|error| match error {
            crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(cause)=>RelationalRecoveryStop::Request(cause),
            crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Content(cause) => cause.into(),
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

impl EnvelopeState {
    pub(super) fn remember_original_failure(&mut self, stop: &RelationalRecoveryStop) {
        if let RelationalRecoveryStop::Durability(error) = stop
            && let Some(failure) = error.storage_unavailable
            && failure.cause.is_proven_damage()
        {
            self.damage = Some(failure);
        }
    }
}
