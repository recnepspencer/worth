//! Original-request control for the SAME positioned authoritative payload grammar.
use super::RelationalCanonicalCommitArtifact as Artifact;
use branch::history::commit::payload_commitment::digest_envelope_in_context;
use branch::history::commit::payload_commitment::writer::{in_context, CommitmentWriter};
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

impl Artifact {
    pub(crate) fn canonical_leaf_from_verified_frame(
        bytes: &[u8],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], Stop> {
        let payload = bytes.get(contracts::durability::canonical_commit_artifact::frame::HEADER_LEN..).ok_or_else(|| {
            contracts::durability::data::recovery_errors::DurabilityError::new(
                contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCanonicalArtifact,
                "verified canonical frame lost its payload",
            )
        })?;
        let canonical = contracts::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload(payload)
            .field("canonical", request)
            .map_err(|cause| match cause {
                contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal::Request(cause) => {
                    Stop::Request(cause)
                }
                cause => contracts::durability::data::recovery_errors::DurabilityError::new(
                    contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCanonicalArtifact,
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
    pub fn matches_positioned_payload_for_request(
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
