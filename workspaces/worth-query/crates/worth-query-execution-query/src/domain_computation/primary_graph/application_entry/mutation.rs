use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::ApplicationSchema;

pub use super::super::handler::{CandidateWriter, DecisionReader, HandlerInterruption};
pub use worth_query_execution_lineage::facade::primary_graph::application_entry::mutation::{
    decision_plan, durable_capability, execution_report, failure, home_retirement,
    HandlerExecutionDenial, HandlerFailure, HandlerResult, WorthQueryDecisionPlan,
    WorthQueryMutationHandlerExecutionReport, WorthQueryMutationHandlerProjectionWork,
    WorthQueryMutationHandlerWork, WorthQueryOperationDurableCapability,
    WorthQueryOperationHomeRetirementDenial, WorthQueryOperationHomeRetirementProgress,
};

mod completed_candidate;
mod execution;
pub use execution::PreparedMutationInputs;
pub use completed_candidate::WorthQueryCompletedMutationCandidate;
pub use execution::build_handler_candidate;
pub use execution::MutationHandlerExecutionDenial;

pub trait OperationHandler<Schema, Binding>: std::any::Any + Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn supports_candidate_planning() -> bool
    where
        Self: Sized,
    {
        false
    }

    fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryOperationHomeRetirementDenial> {
        request
            .consult()
            .map_err(WorthQueryOperationHomeRetirementDenial::Execution)
    }

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

    fn prepare_decision(
        &self,
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> WorthQueryDecisionPlan<Binding::Decision, Binding::Denial> {
        WorthQueryDecisionPlan::immediate(self.decide(input, reader))
    }

    #[doc(hidden)]
    fn prepare_decision_owned(
        &self,
        input: &Binding::Input,
        mut reader: DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> WorthQueryDecisionPlan<Binding::Decision, Binding::Denial> {
        self.prepare_decision(input, &mut reader)
    }

    fn build_candidate(
        &self,
        input: &Binding::Input,
        decision: Binding::Decision,
        candidate: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Binding::Result, Binding::Denial>;
}
