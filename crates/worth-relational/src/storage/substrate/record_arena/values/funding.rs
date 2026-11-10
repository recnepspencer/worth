//! Pre-admit one typed extra and its applied or independently copied payload.
use crate::storage::substrate::StorageAllocation;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{
    AuthoritativePatchApplicationDenial, AuthoritativeRecordAspectPatch as Patch,
    AuthoritativeRecordAspectState as State,
};
#[derive(Debug)]
pub(crate) enum ExtraPayloadDenial {
    Request(crate::execution::RelationalExecutionDenialCause),
    Application(AuthoritativePatchApplicationDenial),
}
fn admit<T>(
    request: &ExecutionRequest<'_, '_>,
    current: Option<&State>,
    patch: &Patch,
) -> Result<
    (
        Option<ExecutionMemoryReservation>,
        Option<ExecutionMemoryReservation>,
    ),
    crate::execution::RelationalExecutionDenialCause,
> {
    let overflow = || crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow;
    let bounds = patch
        .application_retained_storage_bounds(current)
        .ok_or_else(overflow)?;
    let bytes = u64::try_from(bounds.retained_bytes)
        .ok()
        .and_then(|n| n.checked_add(StorageAllocation::<T>::layout_bytes()))
        .ok_or_else(overflow)?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
    let scratch = request
        .reserve_retained_memory(u64::try_from(bounds.transient_bytes).map_err(|_| overflow())?)
        .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
    Ok((memory, scratch))
}
pub(super) fn apply<T>(
    request: &ExecutionRequest<'_, '_>,
    current: Option<&State>,
    patch: &Patch,
    build: impl FnOnce(Option<State>, Option<ExecutionMemoryReservation>) -> T,
) -> Result<T, ExtraPayloadDenial> {
    let (mut memory, _scratch) =
        admit::<T>(request, current, patch).map_err(ExtraPayloadDenial::Request)?;
    match patch.apply_to_optional(current) {
        worth_proof::TransitionOutcome::Success(state) => {
            if state.is_none() {
                // apply_to_optional has already dropped its empty tree roots.
                // The actual Extra survives; retain only its fixed carrier bill.
                if let Some(memory) = &mut memory {
                    memory
                        .resize(StorageAllocation::<T>::layout_bytes())
                        .map_err(|denial| {
                            ExtraPayloadDenial::Request(crate::execution::denial::lease_denial(
                                worth_execution::LeaseDenial::MemoryExhausted(denial),
                            ))
                        })?;
                }
            }
            Ok(build(state.map(|a| a.into_parts().into_parts().0), memory))
        }
        worth_proof::TransitionOutcome::Denied(denial) => {
            Err(ExtraPayloadDenial::Application(denial))
        }
    }
}
pub(super) fn copy<T>(
    request: &ExecutionRequest<'_, '_>,
    state: Option<&State>,
    build: impl FnOnce(Option<State>, Option<ExecutionMemoryReservation>) -> T,
) -> Result<T, crate::execution::RelationalExecutionDenialCause> {
    let (memory, _scratch) = admit::<T>(request, state, &Patch::empty())?;
    Ok(build(state.cloned(), memory))
}
