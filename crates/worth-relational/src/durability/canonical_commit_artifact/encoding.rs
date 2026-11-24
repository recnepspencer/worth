use super::RelationalCanonicalCommitArtifact;
use crate::history::data::PositionedCanonicalCommit;

impl RelationalCanonicalCommitArtifact {
    /// The manifest index uses the journal's exact payload grammar. Hashing
    /// borrows the admitted envelope and allocates no intermediate artifact.
    pub(crate) fn positioned_payload_digest(
        commit: &PositionedCanonicalCommit,
    ) -> Result<[u8; 32], rmp_serde::encode::Error> {
        crate::history::commit::payload_commitment::digest_envelope(
            commit.position(),
            commit.envelope(),
        )
    }

    /// Frame one admitted positioned commit. Only Relational holds a
    /// `PositionedCanonicalCommit`, so only the owner issues artifacts.
    ///
    /// Encoding is deterministic: the same commit always yields the same
    /// bytes. Cost: two serializer passes over the envelope (count, emit),
    /// one policy-admitted allocation of header plus payload, and one
    /// SHA-256 pass over the payload.
    #[cfg(test)]
    pub(crate) fn from_positioned(
        commit: &PositionedCanonicalCommit,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, super::RelationalCanonicalCommitArtifactEncodeDenial> {
        super::verified_frame::VerifiedCanonicalCommitFrame::reference_encode(commit, policy)
            .map(Self::from_verified)
    }
}
