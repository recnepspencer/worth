use super::*;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PlanarOperation};
use worth_query_decl::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey,
};

pub struct PlanarHandler;
impl<Schema: TopologySchemaBinding> OperationHandler<Schema, PlanarMutationBinding<Schema>>
    for PlanarHandler
{
    fn decide(
        &self,
        input: &PlanarMutation,
        reader: &mut DecisionReader<'_, '_, '_, Schema, PlanarMutationBinding<Schema>>,
    ) -> HandlerResult<(), PlanarMutationDenial> {
        if let Err(error) = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone()) {
            return HandlerResult::ExecutionDenied(error);
        }
        match &input.operation {
            PlanarOperation::CreateCycle(vertices) => {
                if vertices.len() < 3 {
                    return HandlerResult::DomainDenied(
                        PlanarMutationDenial::CycleNeedsThreeVertices,
                    );
                }
            }
            PlanarOperation::Adjust(adjustments) => {
                for adjustment in adjustments {
                    let entity = match reader
                        .resolve_entity(BodyKey::reference(), adjustment.body_key.clone())
                    {
                        Ok(entity) => entity,
                        Err(error) => return HandlerResult::ExecutionDenied(error),
                    };
                    match reader.field(&entity, PositionY::reference()) {
                        Ok(Some(_)) => {}
                        Ok(None) => {
                            return HandlerResult::DomainDenied(
                                PlanarMutationDenial::MissingCoordinate,
                            )
                        }
                        Err(error) => return HandlerResult::ExecutionDenied(error),
                    }
                }
            }
        }
        HandlerResult::Completed(())
    }
    fn candidate_requirements(
        &self,
        input: &PlanarMutation,
        _: &(),
    ) -> ApplicationCandidateRequirements {
        let (creates, links, writes) = match &input.operation {
            PlanarOperation::CreateCycle(vertices) => (
                vertices.len(),
                vertices.len(),
                vertices.len().saturating_mul(4),
            ),
            PlanarOperation::Adjust(adjustments) => (0, 0, adjustments.len()),
        };
        requirements(creates, links, writes, 8192, input.validator_work)
    }
    fn build_candidate(
        &self,
        input: &PlanarMutation,
        _: (),
        writer: &mut CandidateWriter<'_, Schema, PlanarMutationBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        match author(input, writer) {
            Ok(changed_vertices) => {
                HandlerResult::Completed(PlanarAdjustmentResult { changed_vertices })
            }
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
fn author<Schema: TopologySchemaBinding>(
    input: &PlanarMutation,
    writer: &mut CandidateWriter<'_, Schema, PlanarMutationBinding<Schema>>,
) -> Result<usize, HandlerExecutionDenial> {
    let anchor = writer
        .resolve_entity(BodyKey::reference(), input.scope_key.clone())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .preserve_output(
            worth_query_host::facade::primary_graph::WorthQueryApplicationOutputRole::new("anchor"),
            &anchor,
        )
        .map_err(HandlerExecutionDenial::new)?;
    match &input.operation {
        PlanarOperation::CreateCycle(vertices) => {
            let mut allocated = Vec::with_capacity(vertices.len());
            for vertex in vertices {
                let key = WorthQueryApplicationEntityKey::new(&vertex.body_key)
                    .map_err(HandlerExecutionDenial::new)?;
                let entity = writer
                    .candidate()
                    .create_entity(Body::reference(), key)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .candidate()
                    .initialize_field(&entity, BodyKey::reference(), vertex.body_key.clone())
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .candidate()
                    .initialize_field(&entity, PositionX::reference(), vertex.x)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .candidate()
                    .initialize_field(&entity, PositionY::reference(), vertex.y)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .candidate()
                    .initialize_field(
                        &entity,
                        Length::reference(),
                        worth_query_consumer_values::PositiveLength::new(1).unwrap(),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
                allocated.push(entity);
            }
            for index in 0..allocated.len() {
                writer
                    .candidate()
                    .link(
                        PlanarSuccessor::reference(),
                        format!("successor:{}", vertices[index].body_key),
                        &allocated[index],
                        &allocated[(index + 1) % allocated.len()],
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
            Ok(allocated.len())
        }
        PlanarOperation::Adjust(adjustments) => {
            for adjustment in adjustments {
                let entity = writer
                    .resolve_entity(BodyKey::reference(), adjustment.body_key.clone())
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .candidate()
                    .write_field(&entity, PositionY::reference(), adjustment.replacement_y)
                    .map_err(HandlerExecutionDenial::new)?;
            }
            Ok(adjustments.len())
        }
    }
}
