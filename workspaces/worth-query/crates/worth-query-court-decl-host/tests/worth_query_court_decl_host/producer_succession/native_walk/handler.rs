//! The genuine preserve handler records the computation in its normal decision plan.
use super::*;
use producers::parent::preserve;
pub struct Handler {
    pub original: preserve::Handler,
    pub computation: model::Installed,
}
impl OperationHandler<Schema, preserve::Binding> for Handler {
    fn decide(
        &self,
        input: &preserve::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, preserve::Binding>,
    ) -> HandlerResult<realization::PublicationDecision, Denial> {
        self.original.decide(input, reader)
    }
    fn prepare_decision(
        &self,
        input: &preserve::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, preserve::Binding>,
    ) -> WorthQueryDecisionPlan<realization::PublicationDecision, Denial> {
        let root = match reader.resolve_entity(Key::at("Key"), input.0.key.clone()) {
            Ok(root) => root,
            Err(cause) => {
                return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(cause));
            }
        };
        let prepared = self.computation.prepare(reader, &root);
        let decision = self.original.decide(input, reader);
        match prepared {
            Ok(prepared) => prepared.into_decision_plan(move |completed| match completed {
                Ok(completed) => {
                    completed.complete().unwrap();
                    decision
                }
                Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
            }),
            Err(cause) => WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                HandlerExecutionDenial::new(cause),
            )),
        }
    }
    fn build_candidate(
        &self,
        input: &preserve::Input,
        decision: realization::PublicationDecision,
        writer: &mut CandidateWriter<'_, Schema, preserve::Binding>,
    ) -> HandlerResult<(), Denial> {
        self.original.build_candidate(input, decision, writer)
    }
}
