//! Issue one original frame from genuine producer data, before performed admission.
use contracts::durability::artifact_payload_emission::request::{extend_length, named_to};
use super::payload_wire::ArtifactPayloadRef;
use super::PreparedOriginalCanonicalPayload;
use super::RelationalCanonicalCommitArtifact as Artifact;
use contracts::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
use contracts::durability::canonical_commit_artifact::frame::{
    CanonicalCommitArtifactHeader, HEADER_LEN,
};
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational_contracts::internal_port as contracts;
pub mod denial;
#[cfg(test)]
pub mod size_tests;
pub use denial::MemoryFramePreparationDenial;

impl Artifact {
    /// The position is reserved producer DATA. This issues bytes, not performed proof.
    pub(crate) fn from_memory_publication(
        position: PatchStreamPosition,
        prepared: &PreparedOriginalCanonicalPayload,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, MemoryFramePreparationDenial> {
        let envelope = prepared.envelope();
        let payload = ArtifactPayloadRef::new(position, envelope);
        let mut payload_len = 0;
        named_to(
            &payload,
            |bytes| {
                let next = extend_length(payload_len, bytes).ok_or(Denial::FrameSizeOverflow)?;
                bounded_frame_len(next)?;
                for _ in bytes.chunks(64 * 1024) {
                    request
                        .consult()
                        .map_err(MemoryFramePreparationDenial::Request)?;
                }
                payload_len = next;
                Ok(())
            },
            encoding_failure,
        )?;
        let frame_len = bounded_frame_len(payload_len)?;
        let header = CanonicalCommitArtifactHeader {
            position,
            commit_id: envelope.commit.commit_id,
            digest: prepared.at_position(position),
        };
        let mut buffer = ExecutionRequestArrayBuilder::<u8>::allocate(*request, frame_len)
            .map_err(denial::allocation)?;
        buffer
            .extend_from_slice(
                &header.emit(u64::try_from(payload_len).map_err(|_| Denial::FrameSizeOverflow)?),
            )
            .map_err(denial::allocation)?;
        named_to(
            &payload,
            |bytes| buffer.extend_from_slice(bytes).map_err(denial::allocation),
            encoding_failure,
        )?;
        if buffer.len() != frame_len {
            return Err(Denial::CountEmissionMismatch {
                counted: payload_len,
                emitted: buffer.len().saturating_sub(HEADER_LEN),
            }
            .into());
        }
        let bytes = buffer.seal_immutable_bytes().map_err(denial::allocation)?;
        // These bytes were deterministically emitted from the exact immutable
        // owner whose two leaves were prepared, with count/emission agreement.
        // External bytes still pass the full frame verification door.
        let artifact = Self::from_verified(super::verified_frame::VerifiedCanonicalCommitFrame {
            bytes,
            header,
        });
        request
            .consult()
            .map_err(MemoryFramePreparationDenial::Request)?;
        Ok(artifact)
    }
}
fn encoding_failure(cause: rmp_serde::encode::Error) -> MemoryFramePreparationDenial {
    Denial::PayloadEncoding {
        detail: cause.to_string(),
    }
    .into()
}

fn bounded_frame_len(payload: usize) -> Result<usize, Denial> {
    let observed = HEADER_LEN
        .checked_add(payload)
        .ok_or(Denial::FrameSizeOverflow)?;
    if observed
        > contracts::durability::canonical_commit_artifact::frame_contract::MAXIMUM_ARTIFACT_BYTES
    {
        return Err(Denial::SizeLimit {
            observed,
            maximum: contracts::durability::canonical_commit_artifact::frame_contract::MAXIMUM_ARTIFACT_BYTES,
        });
    }
    Ok(observed)
}
