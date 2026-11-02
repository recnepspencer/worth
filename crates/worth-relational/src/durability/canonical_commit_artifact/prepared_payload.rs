//! Prepared DATA commitment retains the exact immutable producer envelope.
use super::commitment_writer::{in_context, CommitmentWriter};
use super::payload_digest::RelationalCanonicalCommitArtifactDigest as Digest;
use crate::durability::data::RelationalRecoveryStop;
use crate::history::data::{AuthoritativeArtifactEnvelopeRef, CanonicalCommitEnvelope};
use crate::publication::patch::data::PatchStreamPosition;
use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) struct PreparedOriginalCanonicalPayload {
    envelope: Arc<CanonicalCommitEnvelope>,
    canonical_leaf: [u8; 32],
    inputs_leaf: [u8; 32],
}
impl PreparedOriginalCanonicalPayload {
    pub(crate) fn capture(
        envelope: Arc<CanonicalCommitEnvelope>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalRecoveryStop> {
        let (canonical_leaf, inputs_leaf) = in_context(request, |context| {
            envelope.seal_recorded_original_inputs();
            let mut canonical = CommitmentWriter::new(Sha256::new(), context);
            canonical.value(&AuthoritativeArtifactEnvelopeRef::new(&envelope))?;
            let canonical_leaf = canonical.hash.finalize().into();
            let mut inputs = CommitmentWriter::new(Sha256::new(), context);
            inputs.value(&envelope.recorded_original_inputs())?;
            Ok((canonical_leaf, inputs.hash.finalize().into()))
        })?;
        Ok(Self {
            envelope,
            canonical_leaf,
            inputs_leaf,
        })
    }
    pub(crate) fn envelope(&self) -> &Arc<CanonicalCommitEnvelope> {
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
            self.canonical_leaf,
            self.inputs_leaf,
        )
    }
}
