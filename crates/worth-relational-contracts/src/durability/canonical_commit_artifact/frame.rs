//! Fixed-width artifact header: layout, emission and verification. The
//! verification order is cost-ordered and version-first so bytes from any
//! other format version refuse as unknown rather than as malformed.

use super::frame_contract::MAXIMUM_ARTIFACT_BYTES;
use super::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal as Refusal;
use super::payload_digest::RelationalCanonicalCommitArtifactDigest as Digest;
use crate::history::data::coordinates::CommitId;
#[cfg(test)]
use crate::internal_port::durability::canonical_commit_artifact::payload_digest::RelationalCanonicalCommitArtifactDigestOwnerPort as _;
use crate::publication::patch::data::PatchStreamPosition;

pub(super) const MAGIC: [u8; 8] = *b"WRTHRCCA";
pub const FORMAT_VERSION: u16 = 11;
use crate::durability::positioned_frame::{
    PositionedFrameHeader, PositionedFrameRefusal, PositionedFrameSpec,
};
pub use crate::durability::positioned_frame::{DIGEST_OFFSET, HEADER_LEN};

const SPEC: PositionedFrameSpec = PositionedFrameSpec {
    magic: MAGIC,
    version: FORMAT_VERSION,
    maximum_frame: Some(MAXIMUM_ARTIFACT_BYTES),
};
#[cfg(test)]
const LENGTH_OFFSET: usize = HEADER_LEN - 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalCommitArtifactHeader {
    pub position: PatchStreamPosition,
    pub commit_id: CommitId,
    pub digest: Digest,
}

impl CanonicalCommitArtifactHeader {
    /// Header bytes for a payload of `payload_len` bytes. Encoding writes the
    /// digest slot last, once the payload exists, so callers pass a
    /// placeholder digest here and overwrite `DIGEST_OFFSET` afterwards.
    pub fn emit(&self, payload_len: u64) -> [u8; HEADER_LEN] {
        SPEC.emit(
            PositionedFrameHeader {
                first: self.position.0,
                second: self.commit_id.0,
                digest: *self.digest.as_bytes(),
            },
            payload_len,
        )
    }

    /// Verify one complete frame and return its header with the payload it
    /// covers. Nothing after the declared payload is accepted.
    ///
    /// Cost: O(1) header checks, then one SHA-256 pass over the payload.
    pub fn verify(frame: &[u8]) -> Result<(Self, &[u8]), Refusal> {
        let (header, payload) = Self::inspect(frame)?;
        let computed = Digest::of_payload(header.position.0, header.commit_id.0, payload)?;
        header.require_digest(computed)?;
        Ok((header, payload))
    }

    pub fn require_digest(&self, computed: Digest) -> Result<(), Refusal> {
        if computed != self.digest {
            return Err(Refusal::DigestMismatch {
                recorded: self.digest,
                computed,
            });
        }
        Ok(())
    }

    pub fn inspect(frame: &[u8]) -> Result<(Self, &[u8]), Refusal> {
        let (header, payload) = SPEC.inspect(frame).map_err(frame_refusal)?;
        Ok((
            Self {
                position: PatchStreamPosition(header.first),
                commit_id: CommitId(header.second),
                digest: Digest::recorded(header.digest),
            },
            payload,
        ))
    }
}

/// Read the recorded digest before a separately budgeted verification of the
/// actual original frame. This only reads the framing contract.
pub fn recorded_payload_digest(frame: &[u8]) -> Result<[u8; 32], Refusal> {
    CanonicalCommitArtifactHeader::inspect(frame).map(|(header, _)| *header.digest.as_bytes())
}

pub fn validate_frame_version(frame: &[u8]) -> Result<(), Refusal> {
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
        PositionedFrameRefusal::SizeLimit {
            declared_payload,
            maximum_frame,
        } => Refusal::SizeLimit {
            declared_payload,
            maximum_frame,
        },
    }
}

#[test]
fn content_artifact_ceiling_includes_header_and_refuses_before_payload_or_digest() {
    let mut header = CanonicalCommitArtifactHeader {
        position: PatchStreamPosition(1),
        commit_id: CommitId(1),
        digest: Digest::unsealed(),
    }
    .emit((MAXIMUM_ARTIFACT_BYTES - HEADER_LEN) as u64);
    assert!(matches!(
        CanonicalCommitArtifactHeader::verify(&header),
        Err(Refusal::TruncatedPayload { .. })
    ));
    let excessive = (MAXIMUM_ARTIFACT_BYTES - HEADER_LEN + 1) as u64;
    header[LENGTH_OFFSET..].copy_from_slice(&excessive.to_le_bytes());
    assert_eq!(
        CanonicalCommitArtifactHeader::verify(&header).unwrap_err(),
        Refusal::SizeLimit {
            declared_payload: excessive,
            maximum_frame: MAXIMUM_ARTIFACT_BYTES
        }
    );
}

#[test]
fn only_current_artifact_version_is_admitted_before_payload_checks() {
    for version in [10, FORMAT_VERSION + 1] {
        let mut frame = MAGIC.to_vec();
        frame.extend(version.to_le_bytes());
        assert!(matches!(
            CanonicalCommitArtifactHeader::verify(&frame),
            Err(Refusal::UnsupportedVersion(_))
        ));
    }
    let mut current = MAGIC.to_vec();
    current.extend(FORMAT_VERSION.to_le_bytes());
    assert!(matches!(
        CanonicalCommitArtifactHeader::verify(&current),
        Err(Refusal::TruncatedHeader { .. })
    ));
}
