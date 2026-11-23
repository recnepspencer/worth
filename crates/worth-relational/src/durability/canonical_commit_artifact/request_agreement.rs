//! Original-request control for the SAME positioned authoritative payload grammar.
use super::RelationalCanonicalCommitArtifact as Artifact;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::history::commit::payload_commitment::{
    digest_envelope_in_context,
    writer::{in_context, CommitmentWriter},
};
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl Artifact {
    pub(crate) fn canonical_leaf_from_verified_frame(
        bytes: &[u8],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], Stop> {
        let payload = bytes.get(super::frame::HEADER_LEN..).ok_or_else(|| {
            crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCanonicalArtifact,
                "verified canonical frame lost its payload",
            )
        })?;
        let canonical = super::borrowed_payload::BorrowedPayload(payload)
            .field("canonical", request)
            .map_err(|cause| match cause {
                super::RelationalCanonicalCommitArtifactRefusal::Request(cause) => {
                    Stop::Request(cause)
                }
                cause => crate::durability::data::DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCanonicalArtifact,
                    format!("verified canonical commitment: {cause:?}"),
                )
                .into(),
            })?;
        in_context(request, |context| {
            let mut writer = CommitmentWriter::new(Sha256::new(), context);
            writer.bytes(canonical.0)?;
            Ok(writer.hash.finalize().into())
        })
    }
    pub(crate) fn matches_positioned_payload_for_request(
        original: &PositionedCanonicalCommit,
        candidate: &Arc<CanonicalCommitEnvelope>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Stop> {
        in_context(request, |context| {
            context
                .poll()
                .map_err(|cause| Stop::Request(cause.into()))?;
            if Arc::ptr_eq(original.canonical_arc(), candidate) {
                return Ok(true);
            }
            // Both passes debit one context from the original request.
            let expected =
                digest_envelope_in_context(original.position(), original.envelope(), context)?;
            let actual = digest_envelope_in_context(original.position(), candidate, context)?;
            Ok(expected == actual)
        })
    }
}
