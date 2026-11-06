//! Current frame and owner payload admission under the invoking request.
use super::frame::CanonicalCommitArtifactHeader as Header;
use super::payload_wire::{ArtifactPayloadRef, ArtifactPayloadWire};
use super::{
    RelationalCanonicalCommitArtifact as Artifact,
    RelationalCanonicalCommitArtifactRefusal as Refusal,
};
use crate::durability::artifact_canonical_payload::is_canonical_payload_with_original_context;
use crate::durability::migration::ReadmittedCanonicalCommit;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, MapKernelContext};

impl Artifact {
    /// Admit current bytes only: liveness, bounded frame, funded integrity,
    /// original allocation quotes, controlled content/schema decoding, header
    /// agreement, actual positioned meaning, and funded canonical comparison.
    pub fn readmit(
        bytes: &[u8],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        Self::readmit_with_input(bytes, execution, None)
    }

    pub(super) fn readmit_with_input(
        bytes: &[u8],
        execution: &ExecutionRequest<'_, '_>,
        input: Option<ExecutionImmutableBytes>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        let (header, payload) = Header::inspect(bytes)?;
        let mut input = input;
        let mut answer = None;
        let handoff = crate::execution::request_stage::optional_stage(
            execution,
            |context, _| {
                answer = Some((|| {
                    let digest = super::payload_digest::RelationalCanonicalCommitArtifactDigest::of_payload_for_request(
                    header.position.0, header.commit_id.0, payload, context).map_err(digest_denial)?;
                    header.require_digest(digest)?;
                    Self::readmit_payload(&header, payload, execution, context, input.take())
                })());
            },
            |_| 0,
        );
        complete(handoff, answer)
    }

    /// The actual immutable frame already proved its digest. Preserve its
    /// original byte allocation while admitting every decoded owner field.
    pub(crate) fn readmit_framed(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        let mut answer = None;
        let handoff = crate::execution::request_stage::optional_stage(
            execution,
            |context, _| {
                answer = Some(Self::readmit_payload(
                    &self.header,
                    &self.as_bytes()[super::frame::HEADER_LEN..],
                    execution,
                    context,
                    Some(self.bytes.clone()),
                ));
            },
            |_| 0,
        );
        complete(handoff, answer)
    }

    fn readmit_payload(
        header: &Header,
        payload: &[u8],
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        input: Option<ExecutionImmutableBytes>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        let quote = super::decoded_quote::quote(payload, execution)?;
        let canonical = super::decoded_quote::field(
            super::borrowed_payload::BorrowedPayload(payload),
            "canonical",
            execution,
        )?;
        // The original admissions precede every deserialized allocation.
        let (memory, _transient) =
            super::decoded_quote::admit_canonical(quote, canonical, execution, input, 0)?;
        let original_raw = super::decoded_quote::field(
            super::borrowed_payload::BorrowedPayload(payload),
            "original_inputs",
            execution,
        )?;
        let original_image = if original_raw.is_none() {
            None
        } else {
            let image = super::decoded_quote::field(original_raw, "result_image", execution)?;
            if image.is_none() {
                None
            } else {
                Some(super::prepare_schema_image_decode(image, execution)?)
            }
        };
        let mut wire = super::payload_wire::decode_payload(payload, execution)?;
        if let Some(admission) = original_image {
            let image = wire
                .original_inputs
                .as_mut()
                .and_then(|inputs| inputs.result_image.as_mut())
                .ok_or_else(super::borrowed_payload::malformed)?;
            admission.attach(image)?;
        }
        wire.canonical.retain_readmission_memory(memory);
        wire.canonical
            .attach_shared_readmission_memory()
            .map_err(|detail| Refusal::PayloadDecode {
                detail: detail.into(),
            })?;
        check_header_agreement(header, &wire)?;
        let envelope = wire
            .canonical
            .readmit(execution)
            .map_err(Refusal::FieldText)?;
        if let Some(original) = wire.original_inputs {
            if (original.prestate_schema != original.result_schema)
                != original.result_image.is_some()
            {
                return Err(Refusal::NonCanonicalPayload);
            }
            if let Some(image) = &original.result_image {
                let (registry, _) = image
                    .readmit_registry()
                    .map_err(|detail| Refusal::PayloadDecode { detail })?;
                if image.carrier_digest() != original.result_schema
                    || registry.authority_snapshot() != envelope.schema_authority
                    || image.descriptor_semantics_version() != envelope.descriptor_semantics_version
                {
                    return Err(Refusal::NonCanonicalPayload);
                }
            }
            envelope
                .retain_recorded_original_inputs(original)
                .map_err(|detail| Refusal::PayloadDecode {
                    detail: detail.into(),
                })?;
        }
        let readmitted = ReadmittedCanonicalCommit::readmit_current(header.position, envelope)
            .map_err(Refusal::PositionAdmission)?;
        let canonical = ArtifactPayloadRef::new(readmitted.position(), readmitted.envelope());
        if !is_canonical_payload_with_original_context(&canonical, payload, context)
            .map_err(Refusal::Request)?
        {
            return Err(Refusal::NonCanonicalPayload);
        }
        Ok(readmitted.with_verified_journal_payload(*header.digest.as_bytes()))
    }
}

fn digest_denial(cause: super::payload_digest::RequestDigestDenial) -> Refusal {
    match cause {
        super::payload_digest::RequestDigestDenial::Frame(cause) => Refusal::Frame(cause),
        super::payload_digest::RequestDigestDenial::Request(cause) => Refusal::Request(cause),
    }
}
fn complete(
    handoff: Result<(), crate::execution::RelationalExecutionDenialCause>,
    answer: Option<Result<ReadmittedCanonicalCommit, Refusal>>,
) -> Result<ReadmittedCanonicalCommit, Refusal> {
    match (handoff, answer) {
        (Ok(()), Some(answer)) => answer,
        (Ok(()), None) => unreachable!("completed original artifact admission ran its callback"),
        (Err(_), Some(Err(cause))) => Err(cause),
        (Err(cause), Some(Ok(_)) | None) => Err(Refusal::Request(cause)),
    }
}
fn check_header_agreement(header: &Header, wire: &ArtifactPayloadWire<'_>) -> Result<(), Refusal> {
    if header.position != wire.position {
        return Err(Refusal::PositionDisagreement {
            header: header.position,
            payload: wire.position,
        });
    }
    let envelope_commit = wire.canonical.commit_id();
    if header.commit_id != envelope_commit {
        return Err(Refusal::CommitIdDisagreement {
            header: header.commit_id,
            envelope: envelope_commit,
        });
    }
    Ok(())
}
