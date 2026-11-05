//! Prepared retirement custody contains actual native targets, never row IDs as authority.
use {super::super::CandidateWriter, super::super::HandlerResult, crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantMutationTarget};
use crate::domain_computation::primary_graph::vocabulary::operation_body::WorthQueryOperationBodyDenial;
use crate::domain_computation::request_execution::QueryMemoryReservation;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationOperationBodyProgram, ApplicationSchema},
};
use crate::domain_computation::primary_graph::vocabulary::operation_body::retirement::PreparedBodyRetirements;
pub(super) fn apply<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    prepared: PreparedBodyRetirements<S>,
    body: &ApplicationOperationBodyProgram,
    writer: &mut CandidateWriter<'_, S, B>,
) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
    for batch in prepared.batches {
        let rule = &body.retirements()[batch.rule];
        for (owner, selected) in batch.targets {
            if let Err(cause) = writer.body_retire_owned_selection(&owner, &selected, rule) {
                return HandlerResult::ExecutionDenied(super::super::HandlerExecutionDenial::new(
                    cause,
                ));
            }
        }
    }
    HandlerResult::Completed(())
}
