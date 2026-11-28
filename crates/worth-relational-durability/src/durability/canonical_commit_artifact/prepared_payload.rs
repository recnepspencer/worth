//! Prepared DATA commitment retains the exact immutable producer envelope.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;
use branch::history::commit::payload_commitment::writer::{in_context, write_commitment};
use branch::history::data::canonical_commit_envelope::authoritative_artifact_encoding::AuthoritativeArtifactEnvelopeRef;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use contracts::durability::canonical_commit_artifact::payload_digest::{
    RelationalCanonicalCommitArtifactDigest as Digest,
    RelationalCanonicalCommitArtifactDigestOwnerPort as _,
};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::CanonicalDigestId;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

pub struct PreparedOriginalCanonicalPayload {
    envelope: Arc<CanonicalCommitEnvelope>,
    canonical_leaf: CanonicalDigestId,
    inputs_leaf: CanonicalDigestId,
}
impl PreparedOriginalCanonicalPayload {
    pub fn capture(
        envelope: Arc<CanonicalCommitEnvelope>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalRecoveryStop> {
        let (canonical_leaf, inputs_leaf) = in_context(request, |context| {
            envelope.seal_recorded_original_inputs();
            let canonical = write_commitment(Sha256::new(), context, |sink| {
                sink.value(&AuthoritativeArtifactEnvelopeRef::new(&envelope))
            })?;
            let inputs = write_commitment(Sha256::new(), context, |sink| {
                sink.value(&envelope.recorded_original_inputs())
            })?;
            Ok((
                CanonicalDigestId::new(canonical.finalize().into()),
                CanonicalDigestId::new(inputs.finalize().into()),
            ))
        })?;
        Ok(Self {
            envelope,
            canonical_leaf,
            inputs_leaf,
        })
    }
    pub fn envelope(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.envelope
    }
    /// Only the newly reserved scalar position is unknown during preparation.
    /// This fixed-size combination is DATA, never performed/source admission.
    pub(super) fn at_position(&self, position: PatchStreamPosition) -> Digest {
        let mut bytes = [0u8; 9];
        let mut cursor = std::io::Cursor::new(bytes.as_mut_slice());
        rmp_serde::encode::write_named(&mut cursor, &position)
            .expect(/* Proof: PatchStreamPosition serializes its single u64. MessagePack's largest u64 representation occupies nine bytes, and Cursor over that exact mutable slice has no other failure source. */ "a scalar u64 always fits its fixed nine-byte MessagePack buffer");
        let length = cursor.position() as usize;
        Digest::from_leaves(
            position.0,
            self.envelope.commit.commit_id.0,
            Sha256::digest(&bytes[..length]).into(),
            *self.canonical_leaf.bytes(),
            *self.inputs_leaf.bytes(),
        )
    }
}
