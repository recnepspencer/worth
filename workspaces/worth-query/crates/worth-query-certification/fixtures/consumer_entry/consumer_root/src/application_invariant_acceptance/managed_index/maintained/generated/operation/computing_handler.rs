//! The generated producer's own binding prepares its decision through its
//! partitioned computation when a court arms it; otherwise it decides as the
//! shared generated handler does.
use super::super::super::peer_scope::THROUGH_COMPUTATION;
use super::*;

pub struct ComputingHandler(pub computation::Installed);
impl OperationHandler<Catalog, Binding> for ComputingHandler {
    fn prepare_decision(
        &self,
        input: &Generate,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> WorthQueryDecisionPlan<Decision, Denial> {
        if !THROUGH_COMPUTATION.get() {
            return WorthQueryDecisionPlan::immediate(self.decide(input, reader));
        }
        let request = computation::Request {
            directory: input.directory.clone(),
        };
        let prepared = match self.0.prepare(reader, &request) {
            Ok(prepared) => prepared,
            Err(cause) => {
                return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                    HandlerExecutionDenial::new(cause),
                ))
            }
        };
        let decided = Handler.decide(input, reader);
        prepared.into_decision_plan(move |completed| match (completed, decided) {
            (Ok(completed), HandlerResult::Completed(decision)) => match completed.complete() {
                Ok(_) => HandlerResult::Completed(decision),
                Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
            },
            (Err(cause), _) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
            (_, other) => other,
        })
    }
    fn decide(
        &self,
        input: &Generate,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        Handler.decide(input, reader)
    }
    fn build_candidate(
        &self,
        input: &Generate,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Generated, Denial> {
        Handler.build_candidate(input, decision, writer)
    }
}
