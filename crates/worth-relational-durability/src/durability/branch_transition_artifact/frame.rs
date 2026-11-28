//! Fixed-width artifact header: layout, emission and verification. The
//! verification order is cost-ordered and version-first so bytes from any
//! other format version refuse as unknown rather than as malformed.

use contracts::branch::transition::sequence::RelationalBranchTransitionSequence;
use contracts::durability::branch_transition_artifact::frame_refusal::RelationalBranchTransitionArtifactFrameRefusal as Refusal;
use contracts::durability::branch_transition_artifact::payload_digest::{
    RelationalBranchTransitionArtifactDigest as Digest,
    RelationalBranchTransitionArtifactDigestOwnerPort as _,
};
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_relational_contracts::internal_port as contracts;

pub(super) const MAGIC: [u8; 8] = *b"WRTHRBTA";
pub(super) const FORMAT_VERSION: u16 = 3;

use contracts::durability::positioned_frame::{
    PositionedFrameHeader, PositionedFrameRefusal, PositionedFrameSpec,
};
pub(super) use contracts::durability::positioned_frame::{DIGEST_OFFSET, HEADER_LEN};

const SPEC: PositionedFrameSpec = PositionedFrameSpec {
    magic: MAGIC,
    version: FORMAT_VERSION,
    maximum_frame: None,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BranchTransitionArtifactHeader {
    pub(super) sequence: RelationalBranchTransitionSequence,
    pub(super) follows: PatchStreamPosition,
    pub(super) digest: Digest,
}

impl BranchTransitionArtifactHeader {
    /// Header bytes for a payload of `payload_len` bytes. Encoding writes the
    /// digest slot last, once the payload exists, so callers pass a
    /// placeholder digest here and overwrite `DIGEST_OFFSET` afterwards.
    pub(super) fn emit(&self, payload_len: u64) -> [u8; HEADER_LEN] {
        SPEC.emit(
            PositionedFrameHeader {
                first: self.sequence.0,
                second: self.follows.0,
                digest: *self.digest.as_bytes(),
            },
            payload_len,
        )
    }

    /// Verify one complete frame and return its header with the payload it
    /// covers. Nothing after the declared payload is accepted.
    ///
    /// Cost: O(1) header checks, then one SHA-256 pass over the payload.
    pub(super) fn verify(frame: &[u8]) -> Result<(Self, &[u8]), Refusal> {
        let (coordinates, payload) = SPEC.inspect(frame).map_err(frame_refusal)?;
        let header = Self {
            sequence: RelationalBranchTransitionSequence(coordinates.first),
            follows: PatchStreamPosition(coordinates.second),
            digest: Digest::recorded(coordinates.digest),
        };
        let computed = Digest::of_payload(payload);
        if computed != header.digest {
            return Err(Refusal::DigestMismatch {
                recorded: header.digest,
                computed,
            });
        }
        Ok((header, payload))
    }
}

/// Descriptive prefix eligibility, before payload or digest admission.
pub(super) fn validate_frame_version(frame: &[u8]) -> Result<(), Refusal> {
    SPEC.validate_version(frame).map_err(frame_refusal)
}

fn frame_refusal(cause: PositionedFrameRefusal) -> Refusal {
    match cause {
        PositionedFrameRefusal::TruncatedHeader { available } => {
            Refusal::TruncatedHeader { available }
        }
        PositionedFrameRefusal::UnrecognizedMagic => Refusal::UnrecognizedMagic,
        PositionedFrameRefusal::InvalidVersion => Refusal::InvalidVersion,
        PositionedFrameRefusal::UnsupportedVersion(cause) => Refusal::UnsupportedVersion(cause),
        PositionedFrameRefusal::TruncatedPayload {
            declared,
            available,
        } => Refusal::TruncatedPayload {
            declared,
            available,
        },
        PositionedFrameRefusal::TrailingBytes {
            declared,
            available,
        } => Refusal::TrailingBytes {
            declared,
            available,
        },
        PositionedFrameRefusal::SizeLimit { .. } => {
            unreachable!("transition frames have no inner size limit")
        }
    }
}
