use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
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

/// Refusal to run a mutation handler for an admitted operation.
///
/// No candidate was produced and nothing was committed. A domain denial,
/// cancellation, or deadline from the handler is not here: it arrives inside
/// the `Ok` `HandlerResult`.
#[derive(Debug)]
pub enum MutationHandlerExecutionDenial {
    /// The projection that feeds the handler's decision could not be read.
    Projection(WorthQueryOperationProjectionDenial),
    /// The read attempt, resource reservation, or candidate program failed.
    Attempt(WorthQueryApplicationAttemptDenial),
    /// The handler returned `ExecutionDenied`.
    Handler(HandlerExecutionDenial),
    /// The binding is a workflow control step the workflow kernel records
    /// itself; no handler serves it, so this lane refuses it before any read.
    WorkflowControl,
    /// The admission governed a different input than the one the request
    /// carries, so no handler ran on it.
    InputNotAdmitted,
    /// This runtime installed no handler for the binding, so no handler ran.
    HandlerNotInstalled,
}

impl std::fmt::Display for MutationHandlerExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
            Self::Handler(denial) => denial.fmt(formatter),
            Self::WorkflowControl => formatter.write_str(
                "the workflow kernel records this control binding; no mutation handler serves it",
            ),
            Self::InputNotAdmitted => formatter
                .write_str("the admission governed a different input than the request carries"),
            Self::HandlerNotInstalled => {
                formatter.write_str("this runtime installed no handler for the mutation binding")
            }
        }
    }
}

impl std::error::Error for MutationHandlerExecutionDenial {}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Runs the installed handler for one admitted request and builds its candidate.
    ///
    /// `identities` hold the request's key and input together with the identities
    /// encoded from exactly those two values, so the handler decides on the very
    /// input whose identity the candidate program records; no other input can be
    /// paired with them. The handler reads the key, input and identities from its
    /// `DecisionReader`, and the commit checks the recorded input identity against
    /// the idempotency binding. A capability admission also fixes the identity of
    /// the input it governed, and a request whose input encodes to another
    /// identity is refused with `InputNotAdmitted` before any read.
    pub fn execute_mutation_handler<Binding>(
        &self,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
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
        if admission
            .governed_input_identity()
            .is_some_and(|admitted| admitted != identities.input_identity())
        {
            return Err(MutationHandlerExecutionDenial::InputNotAdmitted);
        }
        let (handler, installed_ceiling) = self.mutation_handler_for_attempt::<Binding>()?;

        let input = identities.mutation_input();
        let request = admission.publication_request();
        let operation_scope_binding = admission.operation_scope_binding().clone();
        let projected = self
            .mutation_projection
            .project_admitted_operation(&admission, |reader, scope| {
                let mut decision_reader = DecisionReader::<Schema, Binding>::new(
                    reader,
                    scope,
                    principal_identity,
                    &operation_scope_binding,
                    identities,
                    request,
                );
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
                        program.bind_mutation_handler_input(identities),
                        result,
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
