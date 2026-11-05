//! Retained target authority crosses decision/candidate without a second lookup.
use crate::domain_computation::primary_graph::vocabulary::operation_body::path_value::PreparedBodyPathWrites;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationOperationBodyProgram, ApplicationSchema},
};
use {super::super::CandidateWriter, super::super::HandlerExecutionDenial};
pub(super) fn apply<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    prepared: PreparedBodyPathWrites<S>,
    body: &ApplicationOperationBodyProgram,
    values: Option<&crate::domain_computation::primary_graph::vocabulary::operation_body::operation_body_value::PreparedBodyValues>,
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
