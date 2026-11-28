//! The authoritative artifact projection has one closed, typed allocation quote.
use worth_relational_contracts::internal_port as contracts;
use super::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::wire_projection::EnvelopeProjection;
use contracts::durability::canonical_commit_artifact::frame::CanonicalCommitArtifactFormat;
use contracts::history::data::canonical_record_allocation::{
    CancelledRecordAllocation, CanonicalRecordAllocation,
};
use contracts::symbols::data::symbol::Symbol;

pub(in super::super) fn quote(
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
    format: CanonicalCommitArtifactFormat,
) -> Result<DecodedQuote, Refusal> {
    let mut result = None;
    let mut seen = 0_u8;
    BorrowedPayload(payload).fields(request, |field, value| {
        let bit = match field {
            "position" => 1,
            "canonical" => 2,
            "original_inputs" => 4,
            _ => return Err(Refusal::NonCanonicalPayload),
        };
        if seen & bit != 0 {
            return Err(Refusal::NonCanonicalPayload);
        }
        seen |= bit;
        if field == "canonical" {
            result = Some(canonical(
                value,
                request,
                match format {
                    CanonicalCommitArtifactFormat::Ordinary => EnvelopeProjection::Authoritative,
                    CanonicalCommitArtifactFormat::LegacyMergeCapable => {
                        EnvelopeProjection::LegacyAuthoritative
                    }
                },
            )?);
        }
        Ok(())
    })?;
    if seen != 7 {
        return Err(malformed());
    }
    result.ok_or_else(malformed)
}
pub(in super::super) fn canonical(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    projection: EnvelopeProjection,
) -> Result<DecodedQuote, Refusal> {
    let mut result = DecodedQuote::default();
    result.control = worth_relational_branch::internal_port::history::data::canonical_commit_envelope::control_allocation::CanonicalEnvelopeControlAllocation::allocation_bytes()
        .ok_or_else(malformed)?;
    let mut seen = 0_u64;
    let mut unexpected_legacy_slot = false;
    value.fields(request, |field, value| {
        let Some(index) = FIELDS.iter().position(|expected| *expected == field) else {
            // The old checkpoint wire ignored unknown fields. Its slice-backed
            // Serde IgnoredAny walks borrowed strings/maps without heap growth;
            // fields() has already visited the whole value under this request.
            #[cfg(test)]
            if matches!(projection, EnvelopeProjection::Checkpoint) {
                return Ok(());
            }
            return Err(Refusal::NonCanonicalPayload);
        };
        let bit = 1_u64 << index;
        if seen & bit != 0 {
            return Err(Refusal::NonCanonicalPayload);
        }
        seen |= bit;
        match field {
            // Content's controlled seed admits its own original allocations.
            "content" => {}
            "producer_cascade_retirement" => add(
                &mut result.cascades,
                super::cascades::producer(value, request)?,
            )?,
            "authored_cascade_retirement" => add(
                &mut result.cascades,
                super::cascades::authored(value, request)?,
            )?,
            "authored_cascade_restoration" => add(
                &mut result.cascades,
                super::cascades::restoration(value, request)?,
            )?,
            "coupled_authored_publication" => add(
                &mut result.cascades,
                super::coupled_publication::quote(value, request)?,
            )?,
            "historical_output_selection" => add(
                &mut result.cascades,
                super::historical_selection::quote(value, request)?,
            )?,
            "authority_kind" => result.selection = super::cascades::authority(value, request)?,
            "semantic_result" => result.semantic = super::semantic::quote(value, request)?,
            "merge_execution_authority" | "merge_parent_branches" | "merge_base_commits" => {
                require_retired_merge_slot_empty(field, value, request)?;
                unexpected_legacy_slot |= !matches!(
                    projection,
                    EnvelopeProjection::LegacyAuthoritative | EnvelopeProjection::Checkpoint
                );
            }
            "index_definition_delta" => result.index_delta = super::indexes::delta(value, request)?,
            "authored_index_writes" => {
                result.index_writes = super::indexes::authored(value, request)?
            }
            "caller_association" => result.association = association(value, request)?,
            "required_output_origin" => {
                result.origin = optional(value, |value| {
                    super::required_output::origin(value, request)
                })?
            }
            _ => add(
                &mut result.envelope,
                field_allocation(field, value, request, projection)?,
            )?,
        }
        Ok(())
    })?;
    if unexpected_legacy_slot {
        return Err(Refusal::NonCanonicalPayload);
    }
    Ok(result)
}

fn require_retired_merge_slot_empty(
    field: &str,
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    if field == "merge_execution_authority" {
        return if value.is_none() {
            Ok(())
        } else {
            Err(Refusal::LegacyMergeArtifactRetired)
        };
    }
    value.elements(request, |_| Err(Refusal::LegacyMergeArtifactRetired))?;
    Ok(())
}

#[test]
fn populated_legacy_merge_slots_refuse_before_deserialization() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let request = ExecutionRequest::serial(&host);
    let absent_authority = rmp_serde::to_vec_named(&Option::<u64>::None).unwrap();
    let present_authority = rmp_serde::to_vec_named(&Some(7_u64)).unwrap();
    let empty_coordinates = rmp_serde::to_vec_named(&[0_u64; 0]).unwrap();
    let present_coordinates = rmp_serde::to_vec_named(&[7_u64]).unwrap();
    for (field, ordinary, merge) in [
        (
            "merge_execution_authority",
            &absent_authority,
            &present_authority,
        ),
        (
            "merge_parent_branches",
            &empty_coordinates,
            &present_coordinates,
        ),
        (
            "merge_base_commits",
            &empty_coordinates,
            &present_coordinates,
        ),
    ] {
        assert!(
            require_retired_merge_slot_empty(field, BorrowedPayload(ordinary), &request).is_ok()
        );
        assert!(matches!(
            require_retired_merge_slot_empty(field, BorrowedPayload(merge), &request),
            Err(Refusal::LegacyMergeArtifactRetired)
        ));
    }
}

fn field_allocation(
    field: &str,
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    projection: EnvelopeProjection,
) -> Result<OwnerAllocation, Refusal> {
    Ok(match field {
        "strategy_artifacts" if value.is_none() => OwnerAllocation::default(),
        "strategy_artifacts" => return Err(Refusal::NonCanonicalPayload),
        "index_definition_delta" => super::indexes::delta(value, request)?,
        "authored_index_writes" => super::indexes::authored(value, request)?,
        "required_output_codec_retirements" => super::codec_retirements::quote(value, request)?,
        "required_output_materialization" => optional(value, |value| {
            super::required_output::materialization(value, request)
        })?,
        "commit" => super::semantic::receipt(value, request)?,
        "branch_context" => text(value, request)?,
        "branch_cell_checkpoint" => optional(value, |value| super::branches::cell(value, request))?,
        "schema_authority" => super::schema::authority(value, request)?,
        "merged_plan" => super::plan::quote(value, request)?,
        "symbol_texts" => super::symbols::quote(value, request)?,
        "revision_symbol_references" => {
            vector::<Symbol>(value, request, |_| Ok(OwnerAllocation::default()))?
        }
        "record_allocations" => {
            vector::<CanonicalRecordAllocation>(value, request, |_| Ok(OwnerAllocation::default()))?
        }
        "cancelled_record_allocations" => {
            vector::<CancelledRecordAllocation>(value, request, |_| Ok(OwnerAllocation::default()))?
        }
        "patch" => super::patch::quote(value, request)?,
        "descriptive_touches" => super::touches::quote(value, request)?,
        "lineage" => super::lineage::quote(value, request)?,
        "schema_transition" => optional(value, |value| super::schema::transition(value, request))?,
        "schema_continuation_descriptor" => {
            optional(value, |value| super::schema::continuation(value, request))?
        }
        "schema_reconciliation_descriptor" => {
            optional(value, |value| super::schema::reconciliation(value, request))?
        }
        // These are omitted semantic lanes in this authoritative projection.
        // Rejecting nonempty input BEFORE decode agrees with its existing exact
        // canonical re-encoding predicate, never accepting cache/diagnostic data.
        "diagnostics_summary" => {
            if matches!(
                projection,
                EnvelopeProjection::Authoritative | EnvelopeProjection::LegacyAuthoritative
            ) {
                require_empty(value, request, "entries")?;
                OwnerAllocation::default()
            } else {
                super::diagnostics::quote(value, request)?
            }
        }
        "derived_index_artifacts" => {
            require_empty(value, request, "generations")?;
            OwnerAllocation::default()
        }
        // Version/authority descriptors are fixed scalar enum/integer owners.
        "authority_kind" | "schema_version" | "descriptor_semantics_version" => {
            OwnerAllocation::default()
        }
        _ => return Err(Refusal::NonCanonicalPayload),
    })
}
fn require_empty(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    omitted: &str,
) -> Result<(), Refusal> {
    value.fields(request, |field, value| {
        if field == omitted && value.elements(request, |_| Ok(()))? != 0 {
            return Err(Refusal::NonCanonicalPayload);
        }
        Ok(())
    })
}
const FIELDS: [&str; 37] = [
    "content",
    "coupled_authored_publication",
    "historical_output_selection",
    "producer_cascade_retirement",
    "authored_cascade_retirement",
    "authored_cascade_restoration",
    "caller_association",
    "index_definition_delta",
    "semantic_result",
    "authored_index_writes",
    "required_output_codec_retirements",
    "required_output_origin",
    "required_output_materialization",
    "commit",
    "branch_context",
    "branch_cell_checkpoint",
    "authority_kind",
    "strategy_artifacts",
    "merge_execution_authority",
    "merge_parent_branches",
    "merge_base_commits",
    "schema_version",
    "schema_authority",
    "merged_plan",
    "symbol_texts",
    "revision_symbol_references",
    "record_allocations",
    "cancelled_record_allocations",
    "patch",
    "descriptive_touches",
    "diagnostics_summary",
    "lineage",
    "derived_index_artifacts",
    "schema_transition",
    "schema_continuation_descriptor",
    "schema_reconciliation_descriptor",
    "descriptor_semantics_version",
];
