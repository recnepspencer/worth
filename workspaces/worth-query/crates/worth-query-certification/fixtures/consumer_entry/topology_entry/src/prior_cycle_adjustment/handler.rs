use super::*;
use worth_query_decl::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
};

mod candidate;
pub(super) mod decision;

pub struct PriorCycleAdjustmentHandler;

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, PriorCycleAdjustmentBinding<Schema>>
    for PriorCycleAdjustmentHandler
{
    fn decide(
        &self,
        input: &PriorCycleAdjustment,
        reader: &mut DecisionReader<'_, '_, '_, Schema, PriorCycleAdjustmentBinding<Schema>>,
    ) -> HandlerResult<decision::PriorCycleDecision<Schema>, PriorCycleAdjustmentDenial> {
        decision::observe_prior_cycle(input, reader)
    }

    fn candidate_requirements(
        &self,
        _: &PriorCycleAdjustment,
        decision: &decision::PriorCycleDecision<Schema>,
    ) -> ApplicationCandidateRequirements {
        super::binding::requirements(decision.members.len())
    }

    fn build_candidate(
        &self,
        input: &PriorCycleAdjustment,
        decision: decision::PriorCycleDecision<Schema>,
        writer: &mut CandidateWriter<'_, Schema, PriorCycleAdjustmentBinding<Schema>>,
    ) -> HandlerResult<PriorCycleAdjustmentResult, PriorCycleAdjustmentDenial> {
        match candidate::adjust_prior_cycle(input, decision, writer) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
