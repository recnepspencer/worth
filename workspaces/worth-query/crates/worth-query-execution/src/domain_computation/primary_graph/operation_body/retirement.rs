//! Prepared retirement custody contains actual native targets, never row IDs as authority.
use super::super::application_contribution::QueryMemoryReservation;
use super::super::{CandidateWriter, HandlerResult, WorthQueryInvariantMutationTarget};
use super::WorthQueryOperationBodyDenial;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationOperationBodyProgram, ApplicationSchema},
};
pub(in crate::domain_computation::primary_graph) struct PreparedRetirementBatch<S> {
    pub(in crate::domain_computation::primary_graph) rule: usize,
    pub(in crate::domain_computation::primary_graph) targets: Vec<(
        WorthQueryInvariantMutationTarget<S, ()>,
        WorthQueryInvariantMutationTarget<S, ()>,
    )>,
    pub(in crate::domain_computation::primary_graph) _memory: QueryMemoryReservation,
}
pub(in crate::domain_computation::primary_graph) struct PreparedBodyRetirements<S> {
    pub(in crate::domain_computation::primary_graph) batches: Vec<PreparedRetirementBatch<S>>,
    pub(in crate::domain_computation::primary_graph) _memory: Option<QueryMemoryReservation>,
}
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
