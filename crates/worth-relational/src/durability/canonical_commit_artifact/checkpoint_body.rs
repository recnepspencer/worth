//! The fixed checkpoint projection uses the same quote, wire and leaf handoff.
use super::{
    borrowed_payload::BorrowedPayload, decoded_quote,
    readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Refusal,
};
use crate::history::data::{CanonicalCommitEnvelope, CanonicalEnvelopeWire, EnvelopeProjection};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

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
        let end = start
            .checked_add(input.len())
            .ok_or_else(super::borrowed_payload::malformed)?;
        let actual = bytes.as_ptr() as usize;
        let actual_end = actual
            .checked_add(bytes.len())
            .ok_or_else(super::borrowed_payload::malformed)?;
        if actual < start || actual_end > end {
            return Err(super::borrowed_payload::malformed());
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
    crate::execution::check_request(request).map_err(Refusal::Request)?;
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
    let mut wire: CanonicalEnvelopeWire<crate::symbols::data::SymbolSnapshotWire<'_>> =
        rmp_serde::from_slice(bytes).map_err(|error| Refusal::PayloadDecode {
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
#[cfg(test)]
pub(crate) fn copy_original_for_test(
    source: &CanonicalCommitEnvelope,
    request: &ExecutionRequest<'_, '_>,
) -> CanonicalCommitEnvelope {
    let bytes = rmp_serde::to_vec_named(&crate::history::data::EmbeddedCheckpointEnvelopeRef::new(
        source,
    ))
    .expect("the actual original checkpoint projection encodes");
    let input =
        ExecutionImmutableBytes::from_external_bytes(std::sync::Arc::new(bytes.into_boxed_slice()));
    let extra = source
        .derived_index_artifacts
        .clone_header_storage_bytes()
        .ok_or_else(super::borrowed_payload::malformed)
        .expect("the exact derived-index header must be quotable");
    let mut copied = decode(input.bytes(), Some(input.clone()), request, extra)
        .expect("the funded copy preserves the complete original meaning");
    source.restore_fixture_custody(&mut copied);
    copied
}

/// Give a recovery fixture an independently funded original body to corrupt.
#[cfg(test)]
pub(crate) fn mutable_positioned_fixture_body<'subject>(
    positioned: &'subject mut crate::history::data::PositionedCanonicalCommit,
    request: &ExecutionRequest<'_, '_>,
) -> &'subject mut crate::history::data::CanonicalCommitEnvelopeBody {
    positioned.positioned_envelope_mut_for_test(|source| copy_original_for_test(source, request))
}
