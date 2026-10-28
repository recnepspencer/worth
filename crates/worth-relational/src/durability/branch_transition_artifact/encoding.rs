use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer};

use super::frame::{BranchTransitionArtifactHeader, DIGEST_OFFSET, HEADER_LEN};
use super::payload_digest::RelationalBranchTransitionArtifactDigest as Digest;
use super::payload_wire::TransitionPayloadRef;
use super::{RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactEncodeDenial};
use crate::branch::PositionedBranchTransition;
use crate::durability::artifact_payload_emission::{count_payload, emit_payload};

impl RelationalBranchTransitionArtifact {
    /// Frame one positioned transition. Only Relational holds a
    /// `PositionedBranchTransition`, so only the owner issues artifacts.
    ///
    /// Encoding is deterministic: the same transition always yields the same
    /// bytes. Cost: two serializer passes over the small transition record,
    /// one policy-admitted allocation of header plus payload, and one SHA-256
    /// pass over the payload.
    pub(crate) fn from_positioned(
        positioned: &PositionedBranchTransition,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, RelationalBranchTransitionArtifactEncodeDenial> {
        policy.check_live()?;
        let payload = TransitionPayloadRef::new(positioned);
        let payload_len = count_payload(&payload, policy)?;
        let frame_len = HEADER_LEN
            .checked_add(payload_len)
            .ok_or(RelationalBranchTransitionArtifactEncodeDenial::FrameSizeOverflow)?;
        let declared_len = u64::try_from(payload_len)
            .map_err(|_| RelationalBranchTransitionArtifactEncodeDenial::FrameSizeOverflow)?;
        let unsealed = BranchTransitionArtifactHeader {
            sequence: positioned.sequence(),
            follows: positioned.follows(),
            digest: Digest::unsealed(),
        };
        let mut frame = ExecutionByteBuffer::allocate(frame_len, policy)?;
        frame.extend_from_slice(&unsealed.emit(declared_len))?;
        emit_payload(&payload, &mut frame)?;
        if frame.len() != frame_len {
            return Err(
                RelationalBranchTransitionArtifactEncodeDenial::CountEmissionMismatch {
                    counted: payload_len,
                    emitted: frame.len().saturating_sub(HEADER_LEN),
                },
            );
        }
        let digest = Digest::of_payload(&frame.bytes()[HEADER_LEN..]);
        frame.overwrite(DIGEST_OFFSET, digest.as_bytes())?;
        Ok(Self {
            bytes: frame.seal()?,
            header: BranchTransitionArtifactHeader { digest, ..unsealed },
        })
    }
}
