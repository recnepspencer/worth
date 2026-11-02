use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer};

use super::frame::{CanonicalCommitArtifactHeader, DIGEST_OFFSET, HEADER_LEN};
use super::payload_digest::RelationalCanonicalCommitArtifactDigest as Digest;
use super::payload_wire::ArtifactPayloadRef;
use super::{RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactEncodeDenial};
use crate::durability::artifact_payload_emission::{count_payload, emit_payload};
use crate::history::data::PositionedCanonicalCommit;

impl RelationalCanonicalCommitArtifact {
    pub(crate) fn positioned_payload_digest_original(
        commit: &PositionedCanonicalCommit,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], RelationalCanonicalCommitArtifactEncodeDenial> {
        use sha2::{Digest, Sha256};
        use RelationalCanonicalCommitArtifactEncodeDenial as Denial;
        struct OriginalHashWriter<'request, 'authority> {
            request: worth_execution::ExecutionRequest<'request, 'authority>,
            digest: Sha256,
            stop: Option<Denial>,
        }
        impl std::io::Write for OriginalHashWriter<'_, '_> {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.stop.is_some() {
                    return Err(std::io::Error::other("original digest request stopped"));
                }
                for chunk in bytes.chunks(4096) {
                    let mut stopped = None;
                    let result = crate::execution::request_stage::optional_stage(
                        &self.request,
                        |context, _| {
                            stopped = crate::execution::request_stage::checkpoint(
                                context,
                                chunk.len() as u64,
                            )
                            .err();
                        },
                        |_| 0,
                    );
                    let stop = match stopped {
                        Some(cause) => Some(cause.into()),
                        None => result.err(),
                    };
                    if let Some(cause) = stop {
                        self.stop = Some(Denial::Request(cause));
                        return Err(std::io::Error::other("original digest request stopped"));
                    }
                    self.digest.update(chunk);
                }
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut writer = OriginalHashWriter {
            request: *execution,
            digest: Sha256::new(),
            stop: None,
        };
        let result = rmp_serde::encode::write_named(
            &mut writer,
            &ArtifactPayloadRef::new(commit.position(), commit.envelope()),
        );
        if let Some(stop) = writer.stop {
            return Err(stop);
        }
        result.map_err(|error| Denial::PayloadEncoding {
            detail: error.to_string(),
        })?;
        crate::execution::check_request(execution).map_err(Denial::Request)?;
        Ok(writer.digest.finalize().into())
    }
    /// The manifest index uses the journal's exact payload grammar. Hashing
    /// borrows the admitted envelope and allocates no intermediate artifact.
    pub(crate) fn positioned_payload_digest(
        commit: &PositionedCanonicalCommit,
    ) -> Result<[u8; 32], rmp_serde::encode::Error> {
        Self::envelope_payload_digest(commit.position(), commit.envelope())
    }

    pub(crate) fn envelope_payload_digest(
        position: crate::publication::patch::data::PatchStreamPosition,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
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
        rmp_serde::encode::write_named(&mut writer, &ArtifactPayloadRef::new(position, envelope))?;
        Ok(writer.0.finalize().into())
    }

    pub(crate) fn matches_positioned_payload(
        original: &PositionedCanonicalCommit,
        candidate: &std::sync::Arc<crate::history::data::CanonicalCommitEnvelope>,
    ) -> Result<bool, rmp_serde::encode::Error> {
        if std::sync::Arc::ptr_eq(original.canonical_arc(), candidate) {
            return Ok(true);
        }
        Ok(Self::positioned_payload_digest(original)?
            == Self::envelope_payload_digest(original.position(), candidate)?)
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
        if frame_len > super::MAXIMUM_ARTIFACT_BYTES {
            return Err(RelationalCanonicalCommitArtifactEncodeDenial::SizeLimit {
                observed: frame_len,
                maximum: super::MAXIMUM_ARTIFACT_BYTES,
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
        let digest = Digest::of_payload(&frame.bytes()[HEADER_LEN..]);
        frame.overwrite(DIGEST_OFFSET, digest.as_bytes())?;
        Ok(Self {
            bytes: frame.seal()?,
            header: CanonicalCommitArtifactHeader { digest, ..unsealed },
        })
    }
}
