//! Canonical codec claims retain the original concrete metadata, never a replay.
use super::*;
pub(super) mod admission;
use crate::history::data::{CanonicalCodecRetirementKind, CanonicalRequiredOutputCodecRetirement};
use crate::runtime::RecoveredCheckpointProgramActivationUpdate;
pub(in super::super) use admission::prepare_canonical_readmission;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    vector::<std::sync::Arc<CanonicalRequiredOutputCodecRetirement>>(value, request, |value| {
        event(value, request)
    })
}
fn event(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_root::<CanonicalRequiredOutputCodecRetirement>()?;
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "meaning" => super::required_output::meaning(value, request)?,
                "source" => super::branches::descriptor(value, request)?,
                "retiring" => super::semantic::receipt(value, request)?,
                "kinds" => kinds(value, request)?,
                "activation" => activation(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn kinds(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    // The wire owns Arc<Vec<Kind>>, not Arc<[Kind]>: its actual Vec and Arc
    // header are different concrete storage, with no invented slice layout.
    let mut bytes = shared_root::<Vec<CanonicalCodecRetirementKind>>()?;
    add(
        &mut bytes,
        vector::<CanonicalCodecRetirementKind>(value, request, |value| {
            let mut bytes = OwnerAllocation::default();
            value.fields(request, |field, value| {
                if matches!(field, "source" | "target") {
                    add(&mut bytes, super::contracts::entity(value, request)?)?;
                }
                Ok(())
            })?;
            Ok(bytes)
        })?,
    )?;
    Ok(bytes)
}
fn activation(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_root::<RecoveredCheckpointProgramActivationUpdate>()?;
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "field_locator" => super::aspects::locator(value, request)?,
                "expected" | "replacement" => super::aspects::canonical_value(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
