//! Strategy artifacts retain declared text, bytes and exact locator rosters.
use super::*;
use crate::commit_strategies::data::{CanonicalStrategyInputArtifact, StrategyMergeDescriptor};
use crate::history::data::StrategyReadmissionMemory;
use worth_foundational::facade::AspectFieldLocator;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "canonical_input" => input(value, request)?,
                "merge_descriptor" => merge(value, request)?,
                "replay_descriptor" => replay(value, request)?,
                // Lowering provenance, summaries, preview validation and cost,
                // validation commit/version axes contain only fixed scalars.
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn input(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "schema_name" | "artifact_name" => text(value, request)?,
                "canonical_bytes" => input_storage(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn merge(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "semantic_name" | "family_name" | "intent_name" => text(value, request)?,
                "intent_scope_targets" => target_storage(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn replay(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if matches!(field, "input_schema_name" | "output_schema_name") {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}

fn input_storage(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_vector::<u8>(value, request, |_| Ok(OwnerAllocation::default()))?;
    add(
        &mut bytes,
        CanonicalStrategyInputArtifact::canonical_bytes_control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
fn target_storage(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_vector::<AspectFieldLocator>(value, request, |value| {
        super::aspects::locator(value, request)
    })?;
    add(
        &mut bytes,
        StrategyMergeDescriptor::intent_scope_targets_control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}

/// Split only the two shared allocations that can survive a bundle clone.
/// Names/other inline-owned values remain in the envelope's actual heap grant.
pub(in super::super) fn prepare_canonical_readmission(
    canonical: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(Option<StrategyReadmissionMemory>, u64), Refusal> {
    let strategy = canonical.optional_field("strategy_artifacts", request)?;
    if strategy.is_none() {
        return Ok((None, 0));
    }
    let input = field(
        field(strategy, "canonical_input", request)?,
        "canonical_bytes",
        request,
    )?;
    let targets = field(
        field(strategy, "merge_descriptor", request)?,
        "intent_scope_targets",
        request,
    )?;
    let input = input_storage(input, request)?.retained;
    let targets = target_storage(targets, request)?.retained;
    let retained = input.checked_add(targets).ok_or_else(malformed)?;
    // Both admissions precede Serde's allocation. Refusal drops only these
    // original reservations; no decoded owner has yet been constructed.
    let input = request
        .reserve_retained_memory(input)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let targets = request
        .reserve_retained_memory(targets)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    Ok((Some(StrategyReadmissionMemory { input, targets }), retained))
}
