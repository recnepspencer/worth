use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::CheckpointBindingRecordFrameLength;

use crate::artifact::durable_frame_rejection::wrong_scope;
use crate::validation::{
    PhysicalArtifactScope, PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

use super::record_rejection::checkpoint_record_denial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointBindingFrameLengthProjection {
    encoded_bytes: u64,
}

impl CheckpointBindingFrameLengthProjection {
    pub const fn encoded_bytes(self) -> u64 {
        self.encoded_bytes
    }
}

pub fn project_checkpoint_binding_frame_length(
    prefix: UntrustedPhysicalArtifact<'_>,
    scope: PhysicalArtifactScope,
) -> Result<CheckpointBindingFrameLengthProjection, PhysicalIntegrityRejection> {
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::CheckpointBinding {
        return Err(wrong_scope(scope));
    }
    let frame = CheckpointBindingRecordFrameLength::decode_prefix(prefix.bytes())
        .map_err(|denial| checkpoint_record_denial(scope, prefix.bytes(), denial))?;
    Ok(CheckpointBindingFrameLengthProjection {
        encoded_bytes: frame.encoded_bytes() as u64,
    })
}
