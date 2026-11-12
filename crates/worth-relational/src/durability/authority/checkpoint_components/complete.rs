//! Controlled exact component DATA/custody; never mints operational basis.
mod data;
use super::*;
use crate::branch::{CheckpointBranchMetadata, CheckpointComponentDescriptorView};
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Stop;
use crate::history::retention::{
    RelationalBranchRetentionBinding, RelationalObservationRetentionObligation,
    RelationalRetentionRequestAcquisitionDenial,
};
pub(in crate::durability) use data::{CapturedCompleteComponents, CompleteComponentHolds};
use std::cmp::Ordering;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMapDenial, ExecutionRequest};
struct Current {
    root: Arc<RelationalBranchRoot>,
    binding: RelationalBranchRetentionBinding,
}
#[derive(Clone, Copy)]
struct Observation<'a> {
    descriptor: CheckpointComponentDescriptorView<'a>,
    root: &'a Arc<RelationalBranchRoot>,
}
fn reserve<T>(
    request: &ExecutionRequest<'_, '_>,
    count: usize,
) -> Result<Option<ExecutionMemoryReservation>, Stop> {
    let layout = std::alloc::Layout::array::<T>(count).map_err(|_| overflow())?;
    let bytes = u64::try_from(layout.size()).map_err(|_| overflow())?;
    request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Stop::Request(cause.into()))
}
fn retention(cause: RelationalRetentionRequestAcquisitionDenial) -> Stop {
    match cause {
        RelationalRetentionRequestAcquisitionDenial::Request(cause) => Stop::Request(cause),
        RelationalRetentionRequestAcquisitionDenial::Storage(
            ExecutionOrderedMapDenial::Request(cause),
        ) => Stop::Request(cause.into()),
        RelationalRetentionRequestAcquisitionDenial::Storage(
            ExecutionOrderedMapDenial::Allocation(cause),
        ) => Stop::Allocation(cause),
        RelationalRetentionRequestAcquisitionDenial::Storage(
            ExecutionOrderedMapDenial::CapacityOverflow,
        ) => overflow(),
        RelationalRetentionRequestAcquisitionDenial::Retention(cause) => {
            use crate::history::retention::RelationalRetentionAcquisitionDenial as Retention;
            let detail = match cause {
                Retention::OwnerUnavailable => return Stop::OwnerUnavailable,
                Retention::CapacityExhausted => return Stop::RetentionCapacityExhausted,
                Retention::IdentityExhausted => return Stop::RetentionIdentityExhausted,
                Retention::RootSetTooLarge => "component retention root set exceeds owner bound",
                Retention::BindingMismatch => {
                    "component retention binding does not match original owner"
                }
            };
            DurabilityError::new(RecoveryFailureClass::ColdStorageUnavailable, detail).into()
        }
    }
}
fn overflow() -> Stop {
    Stop::Request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
fn poll(request: &ExecutionRequest<'_, '_>) -> Result<(), Stop> {
    request
        .consult()
        .map_err(|cause| Stop::Request(cause.into()))
}
/// Caller holds the original configuration read and settled publication epoch.
/// The passed branch DATA is the SAME snapshot captured inside those epochs.
pub(in crate::durability) fn capture_complete(
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
    branch_data: &CheckpointBranchMetadata,
) -> Result<CapturedCompleteComponents, Stop> {
    poll(request)?;
    let _current_memory = reserve::<Current>(request, branch_data.branch_cells.len())?;
    let mut current = Vec::with_capacity(branch_data.branch_cells.len());
    if branch_data.branch_cells.len() != branch_data.branch_lifecycle.len() {
        return Err(corrupt("component branch roster mismatch").into());
    }
    for (cell, life) in branch_data
        .branch_cells
        .iter()
        .zip(branch_data.branch_lifecycle.iter())
    {
        poll(request)?;
        if cell.branch_id != life.branch_id {
            return Err(corrupt("component lifecycle identity mismatch").into());
        }
        let source = runtime
            .history
            .checkpoint_component_source(&cell.branch_id)
            .ok_or_else(|| corrupt("current component disappeared"))?;
        let (root, binding) = source
            .snapshot(cell, life.posture)
            .map_err(|cause| match cause {
                crate::branch::CheckpointBranchMetadataCaptureDenial::Request(cause) => {
                    Stop::Request(cause)
                }
                _ => corrupt("current component observation changed").into(),
            })?
            .ok_or(Stop::OriginalAuthorityUnavailable)?;
        current.push(Current { root, binding });
    }
    let external = runtime
        .history
        .capture_external_checkpoint_components_for_request(
            request,
            current.iter().map(|source| &source.binding),
        )
        .map_err(retention)?;
    let count = current
        .len()
        .checked_add(external.iter().len())
        .ok_or_else(overflow)?;
    let _views_memory = reserve::<Observation<'_>>(request, count)?;
    let mut observations = Vec::with_capacity(count);
    for (index, source) in current.iter().enumerate() {
        poll(request)?;
        let descriptor = CheckpointComponentDescriptorView::current(
            &branch_data.branch_cells[index],
            branch_data.branch_lifecycle[index].posture,
            &source.root,
        )
        .map_err(|_| corrupt("current component root disagrees with original observation"))?;
        observations.push(Observation {
            descriptor,
            root: &source.root,
        });
    }
    for basis in external.iter() {
        poll(request)?;
        let descriptor = CheckpointComponentDescriptorView::retained(basis)
            .map_err(|_| corrupt("retained component root disagrees with original descriptor"))?;
        observations.push(Observation {
            descriptor,
            root: &basis.inner.root,
        });
    }
    observations
        .sort_unstable_by(|left, right| left.descriptor.canonical_wire_cmp(&right.descriptor));
    let mut unique = 0usize;
    let mut bytes = 0usize;
    for (index, observation) in observations.iter().enumerate() {
        poll(request)?;
        if index > 0
            && observations[index - 1]
                .descriptor
                .canonical_wire_cmp(&observation.descriptor)
                == Ordering::Equal
        {
            if !Arc::ptr_eq(observations[index - 1].root, observation.root) {
                return Err(corrupt("one component selects competing original roots").into());
            }
            continue;
        }
        unique = unique.checked_add(1).ok_or_else(overflow)?;
        bytes = bytes
            .checked_add(
                observation
                    .descriptor
                    .allocation_bytes()
                    .ok_or_else(overflow)?,
            )
            .ok_or_else(overflow)?;
    }
    bytes =
        bytes
            .checked_add(
                std::alloc::Layout::array::<(
                    RelationalBranchBasisDescriptor,
                    Arc<RelationalBranchRoot>,
                )>(unique)
                .map_err(|_| overflow())?
                .size(),
            )
            .ok_or_else(overflow)?;
    let memory = request
        .reserve_retained_memory(u64::try_from(bytes).map_err(|_| overflow())?)
        .map_err(|cause| Stop::Request(cause.into()))?;
    let mut rows = Vec::with_capacity(unique);
    for (index, observation) in observations.iter().enumerate() {
        poll(request)?;
        if index > 0
            && observations[index - 1]
                .descriptor
                .canonical_wire_cmp(&observation.descriptor)
                == Ordering::Equal
        {
            continue;
        }
        let descriptor = observation
            .descriptor
            .copy_canonical()
            .map_err(|_| corrupt("component canonical frame refused"))?;
        rows.push((descriptor, Arc::clone(observation.root)));
    }
    let hold_memory = reserve::<RelationalObservationRetentionObligation>(request, current.len())?;
    let mut holds = Vec::with_capacity(current.len());
    for source in &current {
        poll(request)?;
        holds.push(
            RelationalObservationRetentionObligation::acquire_for_request(
                &source.binding,
                Arc::clone(&source.root),
                request,
            )
            .map_err(retention)?,
        );
    }
    Ok(CapturedCompleteComponents::new(
        rows,
        memory,
        CompleteComponentHolds {
            _current: holds,
            _external: external,
            _memory: hold_memory,
        },
    ))
}
#[cfg(test)]
mod tests;
