//! The fixed checkpoint projection uses the same quote, wire and leaf handoff.
use super::{
    borrowed_payload::BorrowedPayload, decoded_quote,
    RelationalCanonicalCommitArtifact as Artifact,
    RelationalCanonicalCommitArtifactRefusal as Refusal,
};
use crate::history::data::{CanonicalCommitEnvelope, CanonicalEnvelopeWire, EnvelopeProjection};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

impl Artifact {
    /// Data readmission only: this neither positions a commit nor grants a
    /// performed publication. The actual checkpoint caller positions it next.
    pub(crate) fn readmit_checkpoint_body(
        bytes: &[u8],
        input: Option<ExecutionImmutableBytes>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CanonicalCommitEnvelope, Refusal> {
        decode(bytes, input, request, 0)
    }
}
impl Artifact {
    pub(crate) fn readmit_embedded_checkpoint(
        bytes: &[u8],
        input: Option<ExecutionImmutableBytes>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::history::data::PositionedCanonicalCommit, Refusal> {
        crate::execution::check_request(request).map_err(Refusal::Request)?;
        require_owned_region(bytes, input.as_ref())?;
        let mut result = None;
        BorrowedPayload(bytes).fields(request, |variant, value| {
            if result.is_some() {
                return Err(super::borrowed_payload::malformed());
            }
            result = Some(match variant {
                "Native" => {
                    let mut seen = 0u8;
                    value.fields(request, |name, _| {
                        let bit = match name {
                            "position" => 1,
                            "canonical" => 2,
                            _ => return Err(super::borrowed_payload::malformed()),
                        };
                        if seen & bit != 0 {
                            return Err(super::borrowed_payload::malformed());
                        }
                        seen |= bit;
                        Ok(())
                    })?;
                    let raw_position = decoded_quote::field(value, "position", request)?;
                    let position = rmp_serde::from_slice::<
                        crate::publication::patch::data::PatchStreamPosition,
                    >(raw_position.0)
                    .map_err(|error| Refusal::PayloadDecode {
                        detail: error.to_string(),
                    })?;
                    let canonical = decoded_quote::field(value, "canonical", request)?;
                    let envelope =
                        Self::readmit_checkpoint_body(canonical.0, input.clone(), request)?;
                    crate::durability::migration::ReadmittedCanonicalCommit::readmit_current(
                        position, envelope,
                    )
                    .map_err(Refusal::PositionAdmission)?
                    .into_positioned()
                }
                "Journal" => {
                    let artifact = value.binary(request)?;
                    Self::readmit_with_input(artifact, request, input.clone())?.into_positioned()
                }
                _ => return Err(super::borrowed_payload::malformed()),
            });
            Ok(())
        })?;
        result.ok_or_else(super::borrowed_payload::malformed)
    }
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
    let mut wire: CanonicalEnvelopeWire<'_> =
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

#[cfg(test)]
impl CanonicalCommitEnvelope {
    /// Ordinary fixture preparation serializes the actual source projection;
    /// that input encoding/import is explicitly outside the funded copy claim.
    /// The same caller request admits every newly decoded/copied allocation.
    pub(crate) fn copy_original_for_test(&self, request: &ExecutionRequest<'_, '_>) -> Self {
        let bytes = rmp_serde::to_vec_named(
            &crate::history::data::EmbeddedCheckpointEnvelopeRef::new(self),
        )
        .expect("the actual original checkpoint projection encodes");
        let input = ExecutionImmutableBytes::from_external_bytes(std::sync::Arc::new(
            bytes.into_boxed_slice(),
        ));
        self.copy_from_checkpoint_input_for_test(&input, request)
            .expect("the funded copy preserves the complete original meaning")
    }

    /// Tests prepare the actual original checkpoint projection as ordinary
    /// caller-owned input, outside this controlled decode/copy operation.
    /// All newly copied backing is admitted before decoder/clone growth.
    pub(crate) fn copy_from_checkpoint_input_for_test(
        &self,
        input: &ExecutionImmutableBytes,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let extra = self
            .derived_index_artifacts
            .clone_header_storage_bytes()
            .ok_or_else(super::borrowed_payload::malformed)?;
        let mut copied = decode(input.bytes(), Some(input.clone()), request, extra)?;
        // im 15.1 OrdMap clones only its pool/root refs; Vector clones only
        // pool refs or its bounded inline Arc elements. No nodes/row payloads
        // are rebuilt. Only this actual new generation roster and two branch
        // strings per generation need new allocation admission.
        copied.body_mut_for_test().derived_index_artifacts = self.derived_index_artifacts.clone();
        if &copied != self {
            return Err(Refusal::NonCanonicalPayload);
        }
        // Preserve only the original owner's existing exact-payload private
        // index proof. Parsed data never creates that live verification.
        copied.body_mut_for_test().authored_index_writes = self.authored_index_writes.clone();
        Ok(copied)
    }
}
