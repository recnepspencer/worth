//! Canonical body admission uses the original funded bytes and expected coordinates.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;
use super::payload_wire::{ArtifactPayloadRef, ArtifactPayloadWire};
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use branch::runtime::state::subsystems::history::canonical_publication_routes::position_admission::readmit_positioned_canonical_commit;
use contracts::durability::artifact_canonical_payload::is_canonical_payload_with_original_context;
use contracts::durability::canonical_commit_artifact::frame::CanonicalCommitArtifactHeader;
use contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Refusal;
use contracts::history::data::coordinates::CommitId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, MapKernelContext};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn readmit_canonical_body(
    header: &CanonicalCommitArtifactHeader,
    payload: &[u8],
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    input: Option<ExecutionImmutableBytes>,
) -> Result<PositionedCanonicalCommit, Refusal> {
    let (position, commit_id) = (header.position, header.commit_id);
    let quote = super::decoded_quote::quote(payload, execution)?;
    let canonical = super::decoded_quote::field(
        contracts::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload(
            payload,
        ),
        "canonical",
        execution,
    )?;
    // The original admissions precede every deserialized allocation.
    let (memory, _transient) =
        super::decoded_quote::admit_canonical(quote, canonical, execution, input, 0)?;
    let original_raw = super::decoded_quote::field(
        contracts::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload(
            payload,
        ),
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
            Some(super::decoded_quote::schema_image::prepare(
                image, execution,
            )?)
        }
    };
    let mut wire = super::payload_wire::decode_payload(payload, execution)?;
    if let Some(admission) = original_image {
        let image = wire
            .original_inputs
            .as_mut()
            .and_then(|inputs| inputs.result_image.as_mut())
            .ok_or_else(
                contracts::durability::canonical_commit_artifact::borrowed_payload::malformed,
            )?;
        admission.attach(image)?;
    }
    wire.canonical.retain_readmission_memory(memory);
    wire.canonical
        .attach_shared_readmission_memory()
        .map_err(|detail| Refusal::PayloadDecode {
            detail: detail.into(),
        })?;
    check_header_agreement(position, commit_id, &wire)?;
    let envelope = wire
        .canonical
        .readmit(execution)
        .map_err(Refusal::FieldText)?;
    if let Some(original) = wire.original_inputs {
        if (original.prestate_schema != original.result_schema) != original.result_image.is_some() {
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
    let readmitted = readmit_positioned_canonical_commit(position, std::sync::Arc::new(envelope))
        .map_err(Refusal::PositionAdmission)?;
    let canonical = ArtifactPayloadRef::new(readmitted.position(), readmitted.envelope());
    if !is_canonical_payload_with_original_context(&canonical, payload, context)
        .map_err(Refusal::Request)?
    {
        return Err(Refusal::NonCanonicalPayload);
    }
    Ok(readmitted)
}

fn check_header_agreement(
    position: PatchStreamPosition,
    commit_id: CommitId,
    wire: &ArtifactPayloadWire<'_>,
) -> Result<(), Refusal> {
    if position != wire.position {
        return Err(Refusal::PositionDisagreement {
            header: position,
            payload: wire.position,
        });
    }
    let envelope_commit = wire.canonical.commit_id();
    if commit_id != envelope_commit {
        return Err(Refusal::CommitIdDisagreement {
            header: commit_id,
            envelope: envelope_commit,
        });
    }
    Ok(())
}

pub(super) fn complete<T>(
    handoff: Result<(), contracts::execution::denial_cause::RelationalExecutionDenialCause>,
    answer: Option<Result<T, Refusal>>,
) -> Result<T, Refusal> {
    match (handoff, answer) {
        (Ok(()), Some(answer)) => answer,
        (Ok(()), None) => unreachable!("completed original artifact admission ran its callback"),
        (Err(_), Some(Err(cause))) => Err(cause),
        (Err(cause), Some(Ok(_)) | None) => Err(Refusal::Request(cause)),
    }
}
