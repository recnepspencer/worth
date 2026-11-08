use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::ApplicationSchema;

pub use super::super::handler::{CandidateWriter, DecisionReader, HandlerInterruption};
mod decision_plan;
mod durable_capability;
pub use decision_plan::WorthQueryDecisionPlan;
pub use durable_capability::WorthQueryOperationDurableCapability;
mod home_retirement;
pub use home_retirement::{
    WorthQueryOperationHomeRetirementDenial, WorthQueryOperationHomeRetirementProgress,
};
mod completed_candidate;
mod execution;
mod execution_report;
mod failure;
pub use completed_candidate::WorthQueryCompletedMutationCandidate;
pub(in crate::domain_computation::primary_graph) use execution::build_handler_candidate;
pub use execution::MutationHandlerExecutionDenial;
pub use execution_report::{
    WorthQueryMutationHandlerExecutionReport, WorthQueryMutationHandlerProjectionWork,
    WorthQueryMutationHandlerWork,
};
pub use failure::HandlerFailure;

/// Execution failure reported by handler-owned decision or candidate work.
#[derive(Debug)]
pub struct HandlerExecutionDenial {
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl HandlerExecutionDenial {
    /// The exact Native stop retained by ordinary or managed-index decision preparation.
    pub fn native_decision_refusal(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::CompanionPreflightStop> {
        execution::native_decision_cause(self.source.as_ref()).or_else(|| {
            crate::domain_computation::primary_graph::invariant_projection::native_index_decision_cause(self.source.as_ref())
        })
    }

    pub fn new(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
        }
    }

    /// Recovers an owning product's concrete handler denial without exposing
    /// that product type to Query.
    pub fn downcast<Denial>(self) -> Result<Denial, Self>
    where
        Denial: std::error::Error + Send + Sync + 'static,
    {
        match self.source.downcast::<Denial>() {
            Ok(denial) => Ok(*denial),
            Err(source) => Err(Self { source }),
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
    /// Admit retirement of the Home's invocation capability for this handler.
    ///
    /// The Home first establishes unique ownership of its original registry
    /// entry and handler Arc. Preflight must preserve business state. After
    /// World seal, that same handler can never be invoked again; its payload
    /// remains retained until explicit, unbudgeted Home reclamation.
    /// Supplier-internal shared state is outside Home custody: close neither
    /// certifies nor destroys it. Override this hook to refuse when the handler
    /// knows that its operational state cannot retire under that boundary.
    fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryOperationHomeRetirementDenial> {
        request
            .consult()
            .map_err(WorthQueryOperationHomeRetirementDenial::Execution)
    }

    /// Complete retirement of the same sealed invocation capability.
    /// The default retains the payload unchanged; it does not certify state
    /// independently shared by the supplier. Overrides retain their original
    /// progress through interruption and must not destroy arbitrary payloads.
    fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryOperationHomeRetirementProgress, WorthQueryOperationHomeRetirementDenial>
    {
        request
            .consult()
            .map_err(WorthQueryOperationHomeRetirementDenial::Execution)?;
        Ok(WorthQueryOperationHomeRetirementProgress::Complete)
    }

    /// Immutable capabilities of this installed handler implementation.
    /// Omission leaves an opaque supplier unknown to directory admission.
    /// Memory-only execution does not impose a durability contract.
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]>
    where
        Self: Sized,
    {
        None
    }

    fn decide(
        &self,
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Binding::Decision, Binding::Denial>;

    /// Prepare every read on the owner and return compute plus owner completion.
    /// A handler without computation keeps its decision implementation unchanged.
    fn prepare_decision(
        &self,
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> WorthQueryDecisionPlan<Binding::Decision, Binding::Denial> {
        WorthQueryDecisionPlan::immediate(self.decide(input, reader))
    }

    fn build_candidate(
        &self,
        input: &Binding::Input,
        decision: Binding::Decision,
        candidate: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Binding::Result, Binding::Denial>;
}
#[cfg(test)]
#[path = "mutation_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
