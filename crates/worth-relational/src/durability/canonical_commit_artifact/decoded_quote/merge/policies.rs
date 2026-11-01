//! Strategy and packet rows retain declared policy/value owners, never authority.
use super::*;
use crate::merge::data::{
    AspectPolicyResolutionRecord, RelationalMergeAdmittedSurfaceRow,
    RelationalMergeAspectPolicyWitnessRow, RelationalMergeDeletionStrategyWitnessRow,
    RelationalMergeTopologyStrategyWitnessRow, ResolvedAspectMergePolicy,
};

pub(super) fn strategy(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "aspect_policy_rows" => {
                    shared_rows::<RelationalMergeAspectPolicyWitnessRow>(value, request, |value| {
                        policy_row(value, request)
                    })?
                }
                "topology_rows" => shared_rows::<RelationalMergeTopologyStrategyWitnessRow>(
                    value,
                    request,
                    |value| posture_row(value, request),
                )?,
                "deletion_rows" => shared_rows::<RelationalMergeDeletionStrategyWitnessRow>(
                    value,
                    request,
                    |value| posture_row(value, request),
                )?,
                "request_digest" | "branch_basis_digest" | "witness_digest" => {
                    text(value, request)?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    // Decode validates, then retains_honest_truth recomputes the witness. The
    // enclosing summary can also perform that same actual honesty comparison.
    bytes.transient = bytes.transient.checked_add(3 * 64).ok_or_else(malformed)?;
    Ok(bytes)
}

pub(super) fn policy_row(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "aspect_resolutions" => {
                    shared_rows::<AspectPolicyResolutionRecord>(value, request, |value| {
                        resolution(value, request)
                    })?
                }
                "applied_policies" => {
                    shared_rows::<ResolvedAspectMergePolicy>(value, request, |value| {
                        identities::policy_declaration(value, request)
                    })?
                }
                "row_digest" => text(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    bytes.transient = bytes.transient.checked_add(3 * 64).ok_or_else(malformed)?;
    Ok(bytes)
}

pub(super) fn resolution(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "aspect_key" => text(value, request)?,
                "applied_policy" => optional(value, |value| identities::policy(value, request))?,
                "resolved_value_strategy" => optional(value, |value| {
                    let mut bytes = OwnerAllocation::default();
                    if value.is_map() {
                        value.fields(request, |variant, value| {
                            if variant == "InlineAspectValue" {
                                add(&mut bytes, aspects::canonical_value(value, request)?)?;
                            }
                            Ok(())
                        })?;
                    }
                    Ok(bytes)
                })?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}

pub(super) fn posture_row(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "row_digest" {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    bytes.transient = 3 * 64;
    Ok(bytes)
}

pub(super) fn packet(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "request" => rows::request(value, request)?,
                "branch_basis" => branches::merge_basis(value, request)?,
                "admitted_merge_surface" => {
                    shared_rows::<RelationalMergeAdmittedSurfaceRow>(value, request, |_| {
                        Ok(OwnerAllocation::default())
                    })?
                }
                "correspondence_witness_digest"
                | "schema_reconciliation_witness_digest"
                | "strategy_witness_digest"
                | "foundational_request_lowering_digest"
                | "admitted_merge_surface_digest"
                | "planning_digest"
                | "execution_digest"
                | "packet_digest" => text(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    // Surface + packet digests, including packet's branch-basis digest.
    bytes.transient = bytes.transient.checked_add(3 * 64).ok_or_else(malformed)?;
    Ok(bytes)
}
