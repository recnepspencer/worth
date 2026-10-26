use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer};

use super::frame::{CanonicalCommitArtifactHeader, DIGEST_OFFSET, HEADER_LEN};
use super::payload_digest::RelationalCanonicalCommitArtifactDigest as Digest;
use super::payload_wire::ArtifactPayloadRef;
use super::{RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactEncodeDenial};
use crate::durability::artifact_payload_emission::{count_payload, emit_payload};
use crate::history::data::PositionedCanonicalCommit;

impl RelationalCanonicalCommitArtifact {
    /// The manifest index uses the journal's exact payload grammar. Hashing
    /// borrows the admitted envelope and allocates no intermediate artifact.
    pub(crate) fn positioned_payload_digest(
        commit: &PositionedCanonicalCommit,
    ) -> Result<[u8; 32], rmp_serde::encode::Error> {
        use sha2::{Digest, Sha256};
        struct HashWriter(Sha256);
        impl std::io::Write for HashWriter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.update(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut writer = HashWriter(Sha256::new());
        rmp_serde::encode::write_named(
            &mut writer,
            &ArtifactPayloadRef::new(commit.position(), commit.envelope()),
        )?;
        Ok(writer.0.finalize().into())
    }

    /// Frame one admitted positioned commit. Only Relational holds a
    /// `PositionedCanonicalCommit`, so only the owner issues artifacts.
    ///
    /// Encoding is deterministic: the same commit always yields the same
    /// bytes. Cost: two serializer passes over the envelope (count, emit),
    /// one policy-admitted allocation of header plus payload, and one
    /// SHA-256 pass over the payload.
    pub(crate) fn from_positioned(
        commit: &PositionedCanonicalCommit,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, RelationalCanonicalCommitArtifactEncodeDenial> {
        policy.check_live()?;
        let payload = ArtifactPayloadRef::new(commit.position(), commit.envelope());
        let payload_len = count_payload(&payload, policy)?;
        let frame_len = HEADER_LEN
            .checked_add(payload_len)
            .ok_or(RelationalCanonicalCommitArtifactEncodeDenial::FrameSizeOverflow)?;
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
        let digest = Digest::of_payload(&frame.bytes()[HEADER_LEN..]);
        frame.overwrite(DIGEST_OFFSET, digest.as_bytes())?;
        Ok(Self {
            bytes: frame.seal()?,
            header: CanonicalCommitArtifactHeader { digest, ..unsealed },
        })
    }
}
