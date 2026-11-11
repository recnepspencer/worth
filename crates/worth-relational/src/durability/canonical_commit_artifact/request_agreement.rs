//! Original-request control for the SAME positioned authoritative payload grammar.
use super::RelationalCanonicalCommitArtifact as Artifact;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext, MapKernelStop, WorkCeilingDenial};

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
        super::commitment_writer::in_context(request, |context| {
            let mut writer =
                super::commitment_writer::CommitmentWriter::new(Sha256::new(), context);
            writer.bytes(canonical.0)?;
            Ok(writer.hash.finalize().into())
        })
    }
    pub(crate) fn matches_positioned_payload_for_request(
        original: &PositionedCanonicalCommit,
        candidate: &Arc<CanonicalCommitEnvelope>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Stop> {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        let mut result = None;
        let scope = request.run_native_with_checkpoint(|context| {
            result = Some((|| {
                context
                    .poll()
                    .map_err(|cause| Stop::Request(cause.into()))?;
                if Arc::ptr_eq(original.canonical_arc(), candidate) {
                    return Ok(true);
                }
                // Both passes debit ONE original context, never a fresh ceiling.
                let expected = digest(original.position(), original.envelope(), context)?;
                let actual = digest(original.position(), candidate, context)?;
                Ok(expected == actual)
            })());
        });
        match (scope, result) {
            (Ok(_), Some(result)) => result,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(Stop::Request(cause.into())),
            (Ok(_), None) => unreachable!("completed original-request agreement callback"),
        }
    }

    pub(crate) fn envelope_payload_digest_for_request(
        position: PatchStreamPosition,
        envelope: &CanonicalCommitEnvelope,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], Stop> {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        let mut result = None;
        let scope = request.run_native_with_checkpoint(|context| {
            result = Some(digest(position, envelope, context));
        });
        match (scope, result) {
            (Ok(_), Some(result)) => result,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(Stop::Request(cause.into())),
            (Ok(_), None) => unreachable!("completed original-request payload callback"),
        }
    }
}

fn digest(
    position: PatchStreamPosition,
    envelope: &CanonicalCommitEnvelope,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<[u8; 32], Stop> {
    use super::commitment_writer::CommitmentWriter;
    let mut position_writer = CommitmentWriter::new(Sha256::new(), context);
    position_writer.value(&position)?;
    let position_leaf = position_writer.hash.finalize().into();
    let mut envelope_writer = CommitmentWriter::new(Sha256::new(), context);
    envelope_writer
        .value(&crate::history::data::AuthoritativeArtifactEnvelopeRef::new(envelope))?;
    let envelope_leaf = envelope_writer.hash.finalize().into();
    let mut input_writer = CommitmentWriter::new(Sha256::new(), context);
    input_writer.value(&envelope.recorded_original_inputs())?;
    let input_leaf = input_writer.hash.finalize().into();
    Ok(
        *super::RelationalCanonicalCommitArtifactDigest::from_leaves_for_request(
            position.0,
            envelope.commit.commit_id.0,
            position_leaf,
            envelope_leaf,
            input_leaf,
            context,
        )
        .map_err(Stop::Request)?
        .as_bytes(),
    )
}
