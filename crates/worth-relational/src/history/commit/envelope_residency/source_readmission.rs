//! Journal and selected singleton bytes share the complete existing admission.
use super::*;
use crate::durability::canonical_commit_artifact::verified_frame::VerifiedCanonicalCommitFrame;
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
        let artifact = VerifiedCanonicalCommitFrame::copy_frame_original(&bytes, execution)
            .map_err(original_frame_stop)?;
        drop(bytes);
        drop(reservation);
        artifact.readmit_indexed(entry, execution)
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

pub(crate) fn original_frame_stop(
    cause: crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial,
) -> RelationalRecoveryStop {
    use crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial as Copy;
    use RelationalRecoveryStop as Stop;
    match cause {
        Copy::Request(cause) => Stop::Request(cause),
        Copy::Allocation(cause) => Stop::Allocation(cause),
        Copy::Frame(cause) => {
            crate::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause)
                .into()
        }
    }
}
