use super::*;

use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
};

mod candidate;
pub(super) mod decision;

pub struct VertexReplacementHandler;
impl<Schema: TopologySchemaBinding> OperationHandler<Schema, VertexReplacementBinding<Schema>>
    for VertexReplacementHandler
{
    fn decide(
        &self,
        input: &VertexReplacement,
        reader: &mut DecisionReader<'_, '_, '_, Schema, VertexReplacementBinding<Schema>>,
    ) -> HandlerResult<decision::ReplacementDecision<Schema>, PlanarReplacementDenial> {
        decision::observe_replacement(input, reader)
    }

    fn build_candidate(
        &self,
        input: &VertexReplacement,
        decision: decision::ReplacementDecision<Schema>,
        writer: &mut CandidateWriter<'_, Schema, VertexReplacementBinding<Schema>>,
    ) -> HandlerResult<PlanarVertexReplacementResult, PlanarReplacementDenial> {
        match candidate::replace_vertex(input, decision, writer) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
