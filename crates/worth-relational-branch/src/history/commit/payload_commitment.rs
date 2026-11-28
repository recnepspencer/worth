//! Canonical payload commitments over positioned envelope DATA.

use worth_relational_contracts::internal_port::durability::canonical_commit_artifact::payload_digest::RelationalCanonicalCommitArtifactDigestOwnerPort as _;
use crate::durability::canonical_commit_artifact::payload_digest::RelationalCanonicalCommitArtifactDigest as Digest;
use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;
use sha2::{Digest as _, Sha256};
use worth_relational_contracts::internal_port::durability::artifact_payload_emission::digest::Sha256Sink;
use worth_execution::{ExecutionRequest, MapKernelContext};

pub mod writer;

pub fn digest_envelope(
    position: PatchStreamPosition,
    envelope: &CanonicalCommitEnvelope,
) -> Result<[u8; 32], rmp_serde::encode::Error> {
    let mut position_writer = Sha256Sink::default();
    rmp_serde::encode::write_named(&mut position_writer, &position)?;
    let mut envelope_writer = Sha256Sink::default();
    rmp_serde::encode::write_named(
        &mut envelope_writer,
        &crate::history::data::AuthoritativeArtifactEnvelopeRef::new(envelope),
    )?;
    let mut input_writer = Sha256Sink::default();
    rmp_serde::encode::write_named(&mut input_writer, &envelope.recorded_original_inputs())?;
    Ok(*Digest::from_leaves(
        position.0,
        envelope.commit.commit_id.0,
        position_writer.0.finalize().into(),
        envelope_writer.0.finalize().into(),
        input_writer.0.finalize().into(),
    )
    .as_bytes())
}

pub fn matches_positioned_payload(
    original: &PositionedCanonicalCommit,
    candidate: &std::sync::Arc<CanonicalCommitEnvelope>,
) -> Result<bool, rmp_serde::encode::Error> {
    if std::sync::Arc::ptr_eq(original.canonical_arc(), candidate) {
        return Ok(true);
    }
    Ok(digest_envelope(original.position(), original.envelope())?
        == digest_envelope(original.position(), candidate)?)
}

pub fn digest_envelope_for_request(
    position: PatchStreamPosition,
    envelope: &CanonicalCommitEnvelope,
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], Stop> {
    writer::in_context(request, |context| {
        digest_envelope_in_context(position, envelope, context)
    })
}

pub fn digest_envelope_in_context(
    position: PatchStreamPosition,
    envelope: &CanonicalCommitEnvelope,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<[u8; 32], Stop> {
    use writer::write_commitment;
    let position_leaf = write_commitment(Sha256::new(), context, |sink| sink.value(&position))?
        .finalize()
        .into();
    let envelope_leaf = write_commitment(Sha256::new(), context, |sink| {
        sink.value(&crate::history::data::AuthoritativeArtifactEnvelopeRef::new(envelope))
    })?
    .finalize()
    .into();
    let input_leaf = write_commitment(Sha256::new(), context, |sink| {
        sink.value(&envelope.recorded_original_inputs())
    })?
    .finalize()
    .into();
    Ok(*Digest::from_leaves_for_request(
        position.0,
        envelope.commit.commit_id.0,
        position_leaf,
        envelope_leaf,
        input_leaf,
        context,
    )
    .map_err(Stop::Request)?
    .as_bytes())
}
