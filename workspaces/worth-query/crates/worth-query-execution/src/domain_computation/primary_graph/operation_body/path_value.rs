//! Retained target authority crosses decision/candidate without a second lookup.
use super::super::{
    application_contribution::QueryMemoryReservation, CandidateWriter, HandlerExecutionDenial,
    WorthQueryInvariantMutationTarget,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationOperationBodyProgram, ApplicationSchema},
};
pub(in crate::domain_computation::primary_graph) struct PreparedBodyPathWrites<S> {
    pub(in crate::domain_computation::primary_graph) targets:
        Vec<WorthQueryInvariantMutationTarget<S, ()>>,
    pub(in crate::domain_computation::primary_graph) _memory: Option<QueryMemoryReservation>,
}
pub(super) fn apply<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    prepared: PreparedBodyPathWrites<S>,
    body: &ApplicationOperationBodyProgram,
    values: Option<&super::value::PreparedBodyValues>,
    writer: &mut CandidateWriter<'_, S, B>,
) -> Result<(), HandlerExecutionDenial> {
    for (step, target) in body.path_value_assignments().iter().zip(prepared.targets) {
        let target = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .body_field(
                &target,
                &step.target.field,
                values
                    .expect("installed computed path value")
                    .value(&step.value),
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}
