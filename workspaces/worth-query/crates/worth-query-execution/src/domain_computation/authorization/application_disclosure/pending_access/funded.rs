//! Original request admission precedes every copied pending-disclosure byte.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure};

/// Governance keeps the pending value and its computation/disclosure copy alive
/// together, so both copies are admitted before the first is constructed.
/// Existing capability authorization remains shared under its original owner.
pub(in crate::domain_computation) fn prepare_funded_pending_disclosure<
    Schema,
    Scope,
    Capability,
    Operation,
    Input,
    E,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    binding: &ApplicationSchemaBindingIdentity,
    principal: EntityId,
    scope: EntityId,
    capability: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    validate_current_request: impl FnOnce() -> Result<(), E>,
    mismatch: impl Fn() -> E,
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    storage_stop: impl Fn(CaptureStop<RetainedStorageDenial>) -> E,
) -> Result<WorthQueryPendingApplicationQueryGovernance, E>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
{
    prepare_with_capture(
        runtime,
        binding,
        principal,
        scope,
        capability,
        validate_current_request,
        mismatch,
        |name, kind, value| {
            let copied = run_retained_stage(execution, memory, |context, memory| {
                checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                let values = value.owned_allocation_capacity_bytes().checked_mul(2);
                let bytes = name
                    .len()
                    .checked_add(kind.len())
                    .and_then(|bytes| bytes.checked_add(values?))
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
                checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
                if let Some(memory) = memory {
                    memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
                }
                Ok((name.to_owned(), kind.to_owned(), value.clone()))
            })
            .map_err(storage_stop)?;
            Ok((copied.0, copied.1, copied.2, ()))
        },
    )
    .map(|(pending, ())| pending)
}
