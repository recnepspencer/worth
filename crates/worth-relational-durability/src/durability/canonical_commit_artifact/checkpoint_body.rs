//! The fixed checkpoint projection uses the same quote, wire and leaf handoff.
use super::decoded_quote;
use branch::history::data::canonical_commit_envelope::fixture_image::CanonicalCommitEnvelopeFixturePort as _;
use branch::history::data::canonical_commit_envelope::text_wire::CanonicalEnvelopeWire;
use branch::history::data::canonical_commit_envelope::wire_projection::EnvelopeProjection;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommitFixturePort as _;
use branch::indexes::data::allocation_accounting::DerivedIndexArtifactsFixturePort as _;
use contracts::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Refusal;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

/// Admit the embedded canonical meaning using the same owned checkpoint body reader.
#[cfg(test)]
pub(crate) fn readmit_checkpoint_body(
    bytes: &[u8],
    input: Option<ExecutionImmutableBytes>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CanonicalCommitEnvelope, Refusal> {
    decode(bytes, input, request, 0)
}

fn require_owned_region(
    bytes: &[u8],
    input: Option<&ExecutionImmutableBytes>,
) -> Result<(), Refusal> {
    if let Some(input) = input {
        let start = input.bytes().as_ptr() as usize;
        let end = start.checked_add(input.len()).ok_or_else(
            contracts::durability::canonical_commit_artifact::borrowed_payload::malformed,
        )?;
        let actual = bytes.as_ptr() as usize;
        let actual_end = actual.checked_add(bytes.len()).ok_or_else(
            contracts::durability::canonical_commit_artifact::borrowed_payload::malformed,
        )?;
        if actual < start || actual_end > end {
            return Err(
                contracts::durability::canonical_commit_artifact::borrowed_payload::malformed(),
            );
        }
    }
    Ok(())
}

fn decode(
    bytes: &[u8],
    input: Option<ExecutionImmutableBytes>,
    request: &ExecutionRequest<'_, '_>,
    additional_envelope_bytes: u64,
) -> Result<CanonicalCommitEnvelope, Refusal> {
    contracts::execution::request_custody::check_request(request).map_err(Refusal::Request)?;
    require_owned_region(bytes, input.as_ref())?;
    let canonical = BorrowedPayload(bytes);
    let quote = decoded_quote::quote_canonical(canonical, request, EnvelopeProjection::Checkpoint)?;
    // Keep every transient admission live until all decoder/check buffers die.
    let (memory, _transient) = decoded_quote::admit_canonical(
        quote,
        canonical,
        request,
        input,
        additional_envelope_bytes,
    )?;
    let mut wire: CanonicalEnvelopeWire<
        contracts::symbols::data::text_wire::SymbolSnapshotWire<'_>,
    > = rmp_serde::from_slice(bytes).map_err(|error| Refusal::PayloadDecode {
        detail: error.to_string(),
    })?;
    wire.retain_readmission_memory(memory);
    wire.attach_shared_readmission_memory()
        .map_err(|detail| Refusal::PayloadDecode {
            detail: detail.into(),
        })?;
    let envelope = wire.readmit(request).map_err(Refusal::FieldText)?;
    // Existing checkpoint admission canonicalizes its typed value owners; it
    // has no artifact-style exact byte-form predicate. Preserve duplicate-map
    // semantics rather than adding a new checkpoint rejection here.
    Ok(envelope)
}

/// Copy a source into a completely readmitted body under the caller's request.
pub fn copy_original_for_test(
    source: &CanonicalCommitEnvelope,
    request: &ExecutionRequest<'_, '_>,
) -> CanonicalCommitEnvelope {
    let bytes = rmp_serde::to_vec_named(
        &branch::history::data::EmbeddedCheckpointEnvelopeRef::new(source),
    )
    .expect("the actual original checkpoint projection encodes");
    let input =
        ExecutionImmutableBytes::from_external_bytes(std::sync::Arc::new(bytes.into_boxed_slice()));
    let extra = source
        .derived_index_artifacts
        .clone_header_storage_bytes_for_test()
        .ok_or_else(contracts::durability::canonical_commit_artifact::borrowed_payload::malformed)
        .expect("the exact derived-index header must be quotable");
    let mut copied = decode(input.bytes(), Some(input.clone()), request, extra)
        .expect("the funded copy preserves the complete original meaning");
    source.restore_fixture_custody_for_test(&mut copied);
    copied
}

/// Give a recovery fixture an independently funded original body to corrupt.
pub fn mutable_positioned_body_for_test<'subject>(
    positioned: &'subject mut branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit,
    request: &ExecutionRequest<'_, '_>,
) -> &'subject mut branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody {
    positioned.positioned_envelope_mut_for_test(|source| copy_original_for_test(source, request))
}
