//! Decision authority, then native link authoring; no application law evaluator.
use super::{operation::*, schema::*};
use worth_query_host::facade::primary_graph::*;

pub struct Handler;
impl<L: LawPolicy> OperationHandler<GraphSchema<L>, EditBinding<L>> for Handler {
    fn decide(
        &self,
        input: &EdgeEdit,
        reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, EditBinding<L>>,
    ) -> HandlerResult<Targets<L>, GraphDenial> {
        let mut targets = Vec::new();
        for &(source, target) in &input.edges {
            let source = match reader.resolve_entity(NodeKey::reference(), source) {
                Ok(value) => value,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
            let target = match reader.resolve_entity(NodeKey::reference(), target) {
                Ok(value) => value,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
            let source = match reader.mutation_target(&source) {
                Ok(value) => value,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
            let target = match reader.mutation_target(&target) {
                Ok(value) => value,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
            targets.push((source, target));
        }
        HandlerResult::Completed(targets)
    }
    fn build_candidate(
        &self,
        input: &EdgeEdit,
        targets: Targets<L>,
        writer: &mut CandidateWriter<'_, GraphSchema<L>, EditBinding<L>>,
    ) -> HandlerResult<Edited, GraphDenial> {
        for (index, (source, target)) in targets.into_iter().enumerate() {
            let source = match writer.projected_entity(&source) {
                Ok(value) => value,
                Err(error) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                }
            };
            let target = match writer.projected_entity(&target) {
                Ok(value) => value,
                Err(error) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                }
            };
            if let Err(error) = writer.link(
                Precedes::reference(),
                format!("edge-{}-{index}", input.serial),
                &source,
                &target,
            ) {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        }
        HandlerResult::Completed(Edited)
    }
}
