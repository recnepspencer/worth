use super::*;
use worth_query_decl::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
};

mod candidate;
mod decision;

pub struct VertexReplacementHandler;
impl<Schema: TopologySchemaBinding> OperationHandler<Schema, VertexReplacementBinding<Schema>>
    for VertexReplacementHandler
{
    fn decide(
        &self,
        input: &VertexReplacement,
        reader: &mut DecisionReader<'_, '_, '_, Schema, VertexReplacementBinding<Schema>>,
    ) -> HandlerResult<(), PlanarReplacementDenial> {
        decision::observe_replacement(input, reader)
    }

    fn candidate_requirements(
        &self,
        _: &VertexReplacement,
        _: &(),
    ) -> ApplicationCandidateRequirements {
        super::binding::replacement_requirements()
    }

    fn build_candidate(
        &self,
        input: &VertexReplacement,
        _: (),
        writer: &mut CandidateWriter<'_, Schema, VertexReplacementBinding<Schema>>,
    ) -> HandlerResult<PlanarVertexReplacementResult, PlanarReplacementDenial> {
        match candidate::replace_vertex(input, writer) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
