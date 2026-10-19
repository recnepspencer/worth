//! Native candidate assembly permits replacement; authored destinations must be distinct.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
use worth_relational::facade::identity::EntityId;
/// O(w log w) work and O(w) temporary slots for all actual authored write targets.
pub(super) fn reject<S>(
    scope: EntityId,
    body: &ApplicationOperationBodyProgram,
    targets: &[WorthQueryInvariantMutationTarget<S, ()>],
    execution: &QueryRequestExecution<'_>,
) -> Result<(), HandlerExecutionDenial> {
    let scope_fields = body
        .scope_fields()
        .iter()
        .filter(|step| step.write_target().is_some());
    let count = scope_fields
        .clone()
        .count()
        .checked_add(targets.len())
        .ok_or_else(overflow)?;
    let bytes = count
        .checked_mul(size_of::<(EntityId, &ApplicationFieldBindingLocus)>())
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(overflow)?;
    let _memory = execution.reserve(bytes).map_err(resource)?;
    execution
        .decision_stage(|context| {
            let work = u64::try_from(count)
                .ok()
                .and_then(|n| n.checked_mul(u64::from(usize::BITS - count.leading_zeros()) + 1))
                .ok_or_else(overflow)?;
            context.checkpoint(work).map_err(|cause| {
                HandlerExecutionDenial::new(advancement_denial(
                    worth_execution::WorkCeilingDenial::Stopped(cause),
                ))
            })?;
            let mut keys = Vec::with_capacity(count);
            keys.extend(scope_fields.map(|step| (scope, step.field())));
            keys.extend(
                body.path_value_assignments()
                    .iter()
                    .zip(targets)
                    .map(|(step, target)| (target.entity_id, &step.target.field)),
            );
            keys.sort_unstable();
            if let Some(pair) = keys.windows(2).find(|pair| pair[0] == pair[1]) {
                return Err(denied(Kind::ConflictingEffectStep, pair[0].1.field()));
            }
            Ok(())
        })
        .map_err(HandlerExecutionDenial::new)?
}
fn overflow() -> HandlerExecutionDenial {
    resource(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
}
