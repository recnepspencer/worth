use super::*;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};

pub struct PlanarInitialAdjustmentHandler;

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, PlanarInitialAdjustmentBinding<Schema>>
    for PlanarInitialAdjustmentHandler
{
    fn decide(
        &self,
        input: &PlanarInitialAdjustment,
        reader: &mut DecisionReader<'_, '_, '_, Schema, PlanarInitialAdjustmentBinding<Schema>>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Body>, PlanarMutationDenial> {
        super::super::source_adjustment::contacts::decision(&observed_input(input));
        let entity = match reader.resolve_entity(BodyKey::reference(), input.scope_key.clone()) {
            Ok(entity) => entity,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&entity, PositionY::reference()) {
            Ok(Some(_)) => {}
            Ok(None) => {
                return HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate)
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&entity) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn build_candidate(
        &self,
        input: &PlanarInitialAdjustment,
        target: WorthQueryInvariantMutationTarget<Schema, Body>,
        writer: &mut CandidateWriter<'_, Schema, PlanarInitialAdjustmentBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        super::super::source_adjustment::contacts::candidate(&observed_input(input));
        let entity = match writer.projected_entity(&target) {
            Ok(entity) => entity,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match writer.write_field(&entity, PositionY::reference(), input.replacement_y) {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }

    fn candidate_requirements(
        &self,
        _: &PlanarInitialAdjustment,
        _: &WorthQueryInvariantMutationTarget<Schema, Body>,
    ) -> ApplicationCandidateRequirements {
        PlanarInitialAdjustmentBinding::<Schema>::CANDIDATES
    }
}

fn observed_input(input: &PlanarInitialAdjustment) -> PlanarSourceAdjustment {
    PlanarSourceAdjustment {
        scope_key: input.scope_key.clone(),
        replacement_y: input.replacement_y,
    }
}
