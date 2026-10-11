use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenialKind as Denial,
    DerivedIndexId,
};

/// Request stops take precedence over the caller's admission at every
/// candidate and completion boundary, including certification traversal.
pub(super) fn admit<E>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    index: DerivedIndexId,
    check_live: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), Stop<E>> {
    execution.consult().map_err(|denial| {
        let cause = match denial {
            worth_execution::WorkCeilingDenial::Admission(denial) => {
                crate::execution::denial::lease_denial(denial)
            }
            worth_execution::WorkCeilingDenial::Stopped(stop) => {
                crate::execution::kernel_failure(worth_execution::MapKernelFailure::Stop(stop))
            }
            worth_execution::WorkCeilingDenial::Panicked => {
                crate::execution::RelationalExecutionDenialCause::WorkerFailed
            }
        };
        Stop::Lookup(super::prepared_entity_field_lookup::denied(
            Denial::Execution(cause),
            index,
        ))
    })?;
    check_live().map_err(Stop::Admission)
}
