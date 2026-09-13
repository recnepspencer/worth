use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_installation::facade::ApplicationSchema;

pub use super::super::handler::{CandidateWriter, DecisionReader, HandlerInterruption};
mod completed_candidate;
mod execution;
pub use completed_candidate::WorthQueryCompletedMutationCandidate;
pub use execution::MutationHandlerExecutionDenial;

/// Execution failure reported by handler-owned decision or candidate work.
#[derive(Debug)]
pub struct HandlerExecutionDenial {
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl HandlerExecutionDenial {
    pub fn new(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
        }
    }
}

impl std::fmt::Display for HandlerExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}

impl std::error::Error for HandlerExecutionDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// Domain outcome of one handler phase, keeping request interruption distinct.
#[derive(Debug)]
pub enum HandlerResult<Value, Denial> {
    Completed(Value),
    DomainDenied(Denial),
    ExecutionDenied(HandlerExecutionDenial),
    Cancelled,
    DeadlineExceeded,
}

/// Execution-owned contract implemented by the handler selected by a binding.
pub trait OperationHandler<Schema, Binding>: Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn decide(
        &self,
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Binding::Decision, Binding::Denial>;

    fn candidate_requirements(
        &self,
        input: &Binding::Input,
        decision: &Binding::Decision,
    ) -> ApplicationCandidateRequirements;

    fn build_candidate(
        &self,
        input: &Binding::Input,
        decision: Binding::Decision,
        candidate: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Binding::Result, Binding::Denial>;
}
