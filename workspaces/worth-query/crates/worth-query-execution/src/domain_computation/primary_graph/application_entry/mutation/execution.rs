use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult,
    WorthQueryCompletedMutationCandidate,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryOperationProjectionDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

#[derive(Debug)]
pub enum MutationHandlerExecutionDenial {
    Projection(WorthQueryOperationProjectionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
    Handler(HandlerExecutionDenial),
}

impl std::fmt::Display for MutationHandlerExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
            Self::Handler(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for MutationHandlerExecutionDenial {}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn execute_mutation_handler<Binding>(
        &self,
        input: &Binding::Input,
        idempotency_key: &Binding::IdempotencyKey,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
    ) -> Result<
        HandlerResult<WorthQueryCompletedMutationCandidate<Schema, Binding>, Binding::Denial>,
        MutationHandlerExecutionDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let (handler, installed_ceiling) = self.mutation_handler_for_attempt::<Binding>();
        let request = admission.publication_request();
        let projected = self
            .mutation_projection
            .project_admitted_operation(&admission, |reader, scope| {
                let mut decision_reader =
                    DecisionReader::<Schema, Binding>::new(reader, scope, idempotency_key, request);
                handler.decide(input, &mut decision_reader)
            })
            .map_err(MutationHandlerExecutionDenial::Projection)?;
        let (decision, projection, _) = projected.into_parts();
        let decision = match decision {
            HandlerResult::Completed(decision) => decision,
            HandlerResult::DomainDenied(denial) => {
                return Ok(HandlerResult::DomainDenied(denial));
            }
            HandlerResult::ExecutionDenied(denial) => {
                return Err(MutationHandlerExecutionDenial::Handler(denial));
            }
            HandlerResult::Cancelled => return Ok(HandlerResult::Cancelled),
            HandlerResult::DeadlineExceeded => return Ok(HandlerResult::DeadlineExceeded),
        };
        let reads = self
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(MutationHandlerExecutionDenial::Attempt)?
            .complete_projected_dependencies()
            .map_err(MutationHandlerExecutionDenial::Attempt)?;
        let requirements = handler.candidate_requirements(input, &decision);
        let mut candidate = reads
            .begin_reserved_effect_program(requirements, installed_ceiling)
            .map_err(MutationHandlerExecutionDenial::Attempt)?;
        candidate
            .prepare_output_contract::<Binding>()
            .map_err(MutationHandlerExecutionDenial::Attempt)?;
        let built = {
            let mut writer = CandidateWriter::<Schema, Binding>::new(&mut candidate);
            handler.build_candidate(input, decision, &mut writer)
        };
        match built {
            HandlerResult::Completed(result) => candidate
                .finish()
                .map(|program| {
                    HandlerResult::Completed(WorthQueryCompletedMutationCandidate::new(
                        program, result,
                    ))
                })
                .map_err(MutationHandlerExecutionDenial::Attempt),
            HandlerResult::DomainDenied(denial) => Ok(HandlerResult::DomainDenied(denial)),
            HandlerResult::ExecutionDenied(denial) => {
                Err(MutationHandlerExecutionDenial::Handler(denial))
            }
            HandlerResult::Cancelled => Ok(HandlerResult::Cancelled),
            HandlerResult::DeadlineExceeded => Ok(HandlerResult::DeadlineExceeded),
        }
    }
}
