//! The authoritative artifact projection has one closed, typed allocation quote.
use super::*;
use crate::history::data::EnvelopeProjection;
use crate::history::data::{BranchId, CanonicalRecordAllocation, CommitId};
use crate::symbols::data::Symbol;

pub(in super::super) fn quote(
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
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
                EnvelopeProjection::Authoritative,
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
    result.control = crate::history::data::CanonicalEnvelopeControlAllocation::allocation_bytes()
        .ok_or_else(malformed)?;
    let mut seen = 0_u64;
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
            "authority_kind" => result.selection = super::cascades::authority(value, request)?,
            "semantic_result" => result.semantic = super::semantic::quote(value, request)?,
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
    Ok(result)
}
fn field_allocation(
    field: &str,
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    projection: EnvelopeProjection,
) -> Result<OwnerAllocation, Refusal> {
    Ok(match field {
        "index_definition_delta" => super::indexes::delta(value, request)?,
        "authored_index_writes" => super::indexes::authored(value, request)?,
        "required_output_codec_retirements" => super::codec_retirements::quote(value, request)?,
        "required_output_materialization" => optional(value, |value| {
            super::required_output::materialization(value, request)
        })?,
        "commit" => super::semantic::receipt(value, request)?,
        "branch_context" => text(value, request)?,
        "branch_cell_checkpoint" => optional(value, |value| super::branches::cell(value, request))?,
        "strategy_artifacts" => optional(value, |value| super::strategy::quote(value, request))?,
        "merge_execution_authority" => {
            optional(value, |value| super::merge::quote(value, request))?
        }
        "merge_parent_branches" => {
            vector::<BranchId>(value, request, |value| text(value, request))?
        }
        "merge_base_commits" => {
            vector::<CommitId>(value, request, |_| Ok(OwnerAllocation::default()))?
        }
        "schema_authority" => super::schema::authority(value, request)?,
        "merged_plan" => super::plan::quote(value, request)?,
        "symbol_texts" => super::symbols::quote(value, request)?,
        "revision_symbol_references" => {
            vector::<Symbol>(value, request, |_| Ok(OwnerAllocation::default()))?
        }
        "record_allocations" => {
            vector::<CanonicalRecordAllocation>(value, request, |_| Ok(OwnerAllocation::default()))?
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
            if matches!(projection, EnvelopeProjection::Authoritative) {
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
const FIELDS: [&str; 34] = [
    "content",
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
