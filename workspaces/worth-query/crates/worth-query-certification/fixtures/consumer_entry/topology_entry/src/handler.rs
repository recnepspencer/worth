use super::*;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PlanarOperation};
use worth_query_decl::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput, WorthQueryCurrentOutputRole, WorthQueryCurrentOutputSelection,
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
            PlanarOperation::RetargetSuccessor {
                source_key,
                previous_target_key,
                replacement_target_key,
            } => {
                let source = match reader.resolve_entity(BodyKey::reference(), source_key.clone()) {
                    Ok(entity) => entity,
                    Err(error) => return HandlerResult::ExecutionDenied(error),
                };
                let previous = match reader
                    .resolve_entity(BodyKey::reference(), previous_target_key.clone())
                {
                    Ok(entity) => entity,
                    Err(error) => return HandlerResult::ExecutionDenied(error),
                };
                if let Err(error) =
                    reader.resolve_entity(BodyKey::reference(), replacement_target_key.clone())
                {
                    return HandlerResult::ExecutionDenied(error);
                }
                match reader.related_one(PlanarSuccessor::reference(), &source) {
                    Ok(target) if target == previous => {}
                    Ok(_) => {
                        return HandlerResult::DomainDenied(
                            PlanarMutationDenial::UnexpectedSuccessor,
                        )
                    }
                    Err(error) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                    }
                }
                match reader.related_one_incoming(PlanarSuccessor::reference(), &previous) {
                    Ok(incoming) if incoming == source => {}
                    Ok(_) => {
                        return HandlerResult::DomainDenied(
                            PlanarMutationDenial::UnexpectedSuccessor,
                        )
                    }
                    Err(error) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                    }
                }
            }
            PlanarOperation::VerifyCurrentOutputs(expectations) => {
                for expectation in expectations {
                    let producer = match reader.resolve_entity(
                        BodyKey::reference(),
                        expectation.producer_key.clone(),
                    ) {
                        Ok(entity) => entity,
                        Err(error) => return HandlerResult::ExecutionDenied(error),
                    };
                    let output = match reader.current_output::<PlanarOutputFamily, Body, Body>(
                        &producer,
                        WorthQueryCurrentOutputRole::new("anchor"),
                    ) {
                        Ok(WorthQueryCurrentOutputSelection::Unique(output)) => output,
                        Ok(WorthQueryCurrentOutputSelection::Missing) => {
                            return HandlerResult::DomainDenied(
                                PlanarMutationDenial::CurrentOutputMissing,
                            )
                        }
                        Ok(WorthQueryCurrentOutputSelection::Ambiguous(_)) => {
                            return HandlerResult::DomainDenied(
                                PlanarMutationDenial::CurrentOutputAmbiguous,
                            )
                        }
                        Ok(WorthQueryCurrentOutputSelection::ObsoleteSource) => {
                            return HandlerResult::DomainDenied(
                                PlanarMutationDenial::CurrentOutputObsolete,
                            )
                        }
                        Err(error) => return HandlerResult::ExecutionDenied(error),
                    };
                    match reader.field(&output, BodyKey::reference()) {
                        Ok(Some(key)) if key == expectation.output_key => {}
                        Ok(_) => {
                            return HandlerResult::DomainDenied(
                                PlanarMutationDenial::UnexpectedCurrentOutput,
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
        let (creates, links, unlinks, writes) = match &input.operation {
            PlanarOperation::CreateCycle(vertices) => (
                vertices.len(),
                vertices.len(),
                0,
                vertices.len().saturating_mul(4),
            ),
            PlanarOperation::Adjust(adjustments) => (0, 0, 0, adjustments.len()),
            PlanarOperation::RetargetSuccessor { .. } => (0, 1, 1, 0),
            PlanarOperation::VerifyCurrentOutputs(_) => (0, 0, 0, 0),
        };
        requirements(creates, links, unlinks, writes, 8192, input.validator_work)
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
            worth_query_host::facade::primary_graph::WorthQueryApplicationOutputRole::from_static("anchor"),
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
                    .create_entity(Body::reference(), key)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .initialize_field(&entity, BodyKey::reference(), vertex.body_key.clone())
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .initialize_field(&entity, PositionX::reference(), vertex.x)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .initialize_field(&entity, PositionY::reference(), vertex.y)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .initialize_field(
                        &entity,
                        Length::reference(),
                        worth_query_consumer_values::PositiveLength::new(1).unwrap(),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
                let role = WorthQueryApplicationOutputRole::<
                    PlanarMutationBinding<Schema>,
                    Body,
                    WorthQueryCreateOutput,
                >::try_new(format!("created.{}", vertex.body_key))
                .map_err(HandlerExecutionDenial::new)?;
                writer
                    .create_output(role, &entity)
                    .map_err(HandlerExecutionDenial::new)?;
                allocated.push(entity);
            }
            for index in 0..allocated.len() {
                writer
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
                    .write_field(&entity, PositionY::reference(), adjustment.replacement_y)
                    .map_err(HandlerExecutionDenial::new)?;
            }
            Ok(adjustments.len())
        }
        PlanarOperation::RetargetSuccessor {
            source_key,
            previous_target_key,
            replacement_target_key,
        } => {
            let source = writer
                .resolve_entity(BodyKey::reference(), source_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            let previous = writer
                .resolve_entity(BodyKey::reference(), previous_target_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            let replacement = writer
                .resolve_entity(BodyKey::reference(), replacement_target_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .unlink(PlanarSuccessor::reference(), &source, &previous)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    PlanarSuccessor::reference(),
                    format!("retarget:{source_key}:{replacement_target_key}"),
                    &source,
                    &replacement,
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(0)
        }
        PlanarOperation::VerifyCurrentOutputs(_) => Ok(0),
    }
}
