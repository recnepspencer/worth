//! One candidate construction tail for ordinary decisions and native body continuations.
use super::super::{CandidateWriter, OperationHandler};
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryProjectedApplicationMutation,
    WorthQueryCompleteApplicationReadSet,
};

pub(in crate::domain_computation::primary_graph) fn build_handler_candidate<Schema, Binding>(
    reads: WorthQueryCompleteApplicationReadSet<
        Schema,
        Binding::Operation,
        Binding::Input,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    handler: &dyn OperationHandler<Schema, Binding>,
    identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
    decision: Binding::Decision,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    HandlerResult<WorthQueryCompletedMutationCandidate<Schema, Binding>, Binding::Denial>,
    MutationHandlerExecutionDenial,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    let mut candidate = reads.begin_effect_program();
    candidate
        .prepare_output_contract::<Binding>()
        .map_err(MutationHandlerExecutionDenial::Attempt)?;
    let built = {
        let mut writer = CandidateWriter::<Schema, Binding>::new(&mut candidate);
        handler.build_candidate(identities.mutation_input(), decision, &mut writer)
    };
    match built {
        HandlerResult::Completed(result) => {
            candidate
                .observe_unique_write_dependencies(execution, allocation)
                .map_err(MutationHandlerExecutionDenial::Attempt)?;
            candidate
                .finish()
                .map(|program| {
                    HandlerResult::Completed(WorthQueryCompletedMutationCandidate::new(
                        program.bind_mutation_handler_input(identities),
                        result,
                    ))
                })
                .map_err(MutationHandlerExecutionDenial::Attempt)
        }
        HandlerResult::DomainDenied(denial) => Ok(HandlerResult::DomainDenied(denial)),
        HandlerResult::ExecutionDenied(denial) => {
            Err(MutationHandlerExecutionDenial::Handler(denial))
        }
        HandlerResult::Cancelled => Ok(HandlerResult::Cancelled),
        HandlerResult::DeadlineExceeded => Ok(HandlerResult::DeadlineExceeded),
    }
}
