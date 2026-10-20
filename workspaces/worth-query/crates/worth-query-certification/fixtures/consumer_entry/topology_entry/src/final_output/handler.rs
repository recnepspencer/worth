use super::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};

pub struct FinalPlanarMutationHandler;

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, FinalPlanarMutationBinding<Schema>>
    for FinalPlanarMutationHandler
{
    fn decide(
        &self,
        input: &FinalPlanarMutation,
        reader: &mut DecisionReader<'_, '_, '_, Schema, FinalPlanarMutationBinding<Schema>>,
    ) -> HandlerResult<
        worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<Schema, Body>,
        worth_query_consumer_values::PlanarMutationDenial,
    > {
        match reader.resolve_entity(BodyKey::reference(), input.scope_key.clone()) {
            Ok(source) => match reader.mutation_target(&source) {
                Ok(target) => HandlerResult::Completed(target),
                Err(error) => HandlerResult::ExecutionDenied(error),
            },
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &FinalPlanarMutation,
        _: &worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<
            Schema,
            Body,
        >,
    ) -> ApplicationCandidateRequirements {
        FinalPlanarMutationBinding::<Schema>::CANDIDATES
            .finite_requirements()
            .expect("fixed-shape handler declares finite candidate policy")
    }

    fn build_candidate(
        &self,
        input: &FinalPlanarMutation,
        source: worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<
            Schema,
            Body,
        >,
        writer: &mut CandidateWriter<'_, Schema, FinalPlanarMutationBinding<Schema>>,
    ) -> HandlerResult<
        worth_query_consumer_values::PlanarAdjustmentResult,
        worth_query_consumer_values::PlanarMutationDenial,
    > {
        let retained = match writer.projected_entity(&source) {
            Ok(entity) => entity,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        for bound in [
            writer.preserve_output::<FinalRetainedSourceOutput<Schema>>(&retained),
            writer.preserve_member::<FinalRetainedSources<Schema>>(&input.scope_key, &retained),
        ] {
            if let Err(error) = bound {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        }
        let keys = [
            input.output_key.clone(),
            format!("{}:b", input.output_key),
            format!("{}:c", input.output_key),
        ];
        let base_y = worth_query_consumer_values::PositiveLength::get(&input.value);
        let Some(next_y) = base_y.checked_add(1) else {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                std::io::Error::other("final output ordinate exceeds its value binding"),
            ));
        };
        let coordinates = [(1, base_y), (2, base_y), (1, next_y)];
        let mut entities = Vec::with_capacity(3);
        for (key, (x, y)) in keys.iter().zip(coordinates) {
            let entity_key =
                match worth_query_host::facade::primary_graph::WorthQueryApplicationEntityKey::new(
                    key,
                ) {
                    Ok(key) => key,
                    Err(error) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                    }
                };
            let entity = match writer.create_entity(Body::reference(), entity_key) {
                Ok(entity) => entity,
                Err(error) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                }
            };
            for result in [
                writer.initialize_field(&entity, BodyKey::reference(), key.clone()),
                writer.initialize_field(
                    &entity,
                    PositionX::reference(),
                    worth_query_consumer_values::PositiveLength::new(x).unwrap(),
                ),
                writer.initialize_field(
                    &entity,
                    PositionY::reference(),
                    worth_query_consumer_values::PositiveLength::new(y).unwrap(),
                ),
                writer.initialize_field(&entity, Length::reference(), input.value),
            ] {
                if let Err(error) = result {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
                }
            }
            entities.push(entity);
        }
        for index in 0..entities.len() {
            if let Err(error) = writer.link(
                PlanarSuccessor::reference(),
                format!("final-successor:{}", keys[index]),
                &entities[index],
                &entities[(index + 1) % entities.len()],
            ) {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        }
        let entity = &entities[0];
        for bound in [
            writer.create_output::<FinalAnchorOutput<Schema>>(entity),
            writer.create_member::<FinalCreatedOutputs<Schema>>(&keys[1], &entities[1]),
            writer.create_output::<FinalClosingOutput<Schema>>(&entities[2]),
        ] {
            if let Err(error) = bound {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        }
        HandlerResult::Completed(worth_query_consumer_values::PlanarAdjustmentResult {
            changed_vertices: 1,
        })
    }
}
