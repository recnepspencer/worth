//! Independent policy-funded wire reference used by artifact and residency tests.
use worth_relational_contracts::internal_port::durability::canonical_commit_artifact::payload_digest::RelationalCanonicalCommitArtifactDigestOwnerPort as _;
use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer};

use super::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial;
use super::frame::{CanonicalCommitArtifactHeader, DIGEST_OFFSET, HEADER_LEN};
use super::frame_contract::MAXIMUM_ARTIFACT_BYTES;
use super::payload_digest::RelationalCanonicalCommitArtifactDigest as Digest;
use super::payload_wire::ArtifactPayloadRef;
use super::verified_frame::VerifiedCanonicalCommitFrame;
use crate::durability::artifact_payload_emission::{count_payload, emit_payload};
use crate::history::data::PositionedCanonicalCommit;

pub(super) fn encode(
    commit: &PositionedCanonicalCommit,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<VerifiedCanonicalCommitFrame, RelationalCanonicalCommitArtifactEncodeDenial> {
    policy.check_live()?;
    let payload = ArtifactPayloadRef::new(commit.position(), commit.envelope());
    let payload_len = count_payload(&payload, policy)?;
    let frame_len = HEADER_LEN
        .checked_add(payload_len)
        .ok_or(RelationalCanonicalCommitArtifactEncodeDenial::FrameSizeOverflow)?;
    if frame_len > MAXIMUM_ARTIFACT_BYTES {
        return Err(RelationalCanonicalCommitArtifactEncodeDenial::SizeLimit {
            observed: frame_len,
            maximum: MAXIMUM_ARTIFACT_BYTES,
        });
    }
    let declared_len = u64::try_from(payload_len)
        .map_err(|_| RelationalCanonicalCommitArtifactEncodeDenial::FrameSizeOverflow)?;
    let unsealed = CanonicalCommitArtifactHeader {
        position: commit.position(),
        commit_id: commit.envelope().commit.commit_id,
        digest: Digest::unsealed(),
    };
    let mut frame = ExecutionByteBuffer::allocate(frame_len, policy)?;
    frame.extend_from_slice(&unsealed.emit(declared_len))?;
    emit_payload(&payload, &mut frame)?;
    if frame.len() != frame_len {
        return Err(
            RelationalCanonicalCommitArtifactEncodeDenial::CountEmissionMismatch {
                counted: payload_len,
                emitted: frame.len().saturating_sub(HEADER_LEN),
            },
        );
    }
    let digest = Digest::canonical_payload_digest_for_test(
        commit.position().0,
        commit.envelope().commit.commit_id.0,
        &frame.bytes()[HEADER_LEN..],
    )
    .map_err(
        |cause| RelationalCanonicalCommitArtifactEncodeDenial::PayloadEncoding {
            detail: cause.to_string(),
        },
    )?;
    frame.overwrite(DIGEST_OFFSET, digest.as_bytes())?;
    Ok(VerifiedCanonicalCommitFrame {
        bytes: frame.seal()?,
        header: CanonicalCommitArtifactHeader { digest, ..unsealed },
    })
}
