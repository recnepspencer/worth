//! Journal and selected singleton bytes share the complete existing admission.
use super::*;
impl RelationalCommitEnvelopeResidency {
    pub(crate) fn original_source_damage(&self) -> Option<RelationalStorageUnavailable> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .damage
    }

    pub(crate) fn remember_original_failure(&self, stop: &RelationalRecoveryStop) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remember_original_failure(stop);
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
        let read_limit = limit.min(
            crate::durability::canonical_commit_artifact::frame_contract::MAXIMUM_ARTIFACT_BYTES
                as u64,
        );
        let reservation = execution
            .reserve_retained_memory(read_limit)
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        let bytes = source
            .read_commit(entry.position, read_limit)
            .map_err(failure)?;
        let artifact = RelationalCanonicalCommitArtifact::copy_frame_original(&bytes, execution)
            .map_err(|cause| match cause {
                crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial::Request(cause) => RelationalRecoveryStop::Request(cause),
                crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial::Allocation(cause) => RelationalRecoveryStop::Allocation(cause),
                crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial::Frame(cause) => crate::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause).into(),
            })?;
        drop(bytes);
        drop(reservation);
        Self::readmit_owned_artifact(entry, artifact, execution)
    }
    pub(crate) fn readmit_owned_artifact(
        entry: &RelationalCommitIndexEntry,
        artifact: RelationalCanonicalCommitArtifact,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        Self::check_frame(entry, &artifact)?;
        let admitted=artifact.readmit_framed(execution).map_err(|error| match error {
            crate::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal::Request(cause)=>RelationalRecoveryStop::Request(cause),
            crate::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal::Content(cause) => cause.into(),
                crate::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal::FieldText(cause)=>crate::symbols::data::field_text_recovery_stop(cause),
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
        crate::execution::request_custody::check_request(execution)
            .map_err(RelationalRecoveryStop::Request)?;
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
