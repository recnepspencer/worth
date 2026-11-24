//! Read original Memory-frame digest metadata without rehashing the body.
use super::source_readmission::original_frame_stop;
use super::{EnvelopeBacking, RelationalCommitEnvelopeResidency};
use crate::durability::canonical_commit_artifact::frame::recorded_payload_digest;
use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;

impl RelationalCommitEnvelopeResidency {
    /// Capture metadata reads the digest the original Memory frame records,
    /// never re-encoding or re-hashing it: the capture verifies each frame
    /// once, when it embeds it, against the digest this returns.
    pub(crate) fn original_memory_recorded_digest(&self) -> Result<[u8; 32], Stop> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let EnvelopeBacking::Memory {
            original: Some(bytes),
            ..
        } = &state.backing
        else {
            return Err(crate::durability::data::recovery_errors::DurabilityError::new(
                crate::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                "original Memory artifact unavailable or damaged",
            )
            .into());
        };
        recorded_payload_digest(bytes.bytes()).map_err(|cause| {
            original_frame_stop(
                crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial::Frame(
                    cause,
                ),
            )
        })
    }
}
