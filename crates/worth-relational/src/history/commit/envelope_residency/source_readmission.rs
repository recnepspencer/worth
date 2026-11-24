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
