use super::*;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PlanarOperation};
use worth_query_decl::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_decl::facade::application_schema::{
    OperationCreates, OperationLinks, OperationReads, OperationUnlinks, OperationWrites,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey, WorthQueryCurrentOutputSelection,
};

/// A planar body keyed with this prefix surveys the length its commit
/// publishes before it decides.
pub(crate) const SURVEYED_BODY_KEY_PREFIX: &str = "surveyed-";

/// The widest set a survey reads before it reads that set again at exactly
/// its number: one standing body and the anchor that joins it. The alternate
/// operation declares a projection budget of 8, which pays for no wider
/// survey beside its other reads.
const SURVEY_WIDTH: usize = 2;

/// Read the bodies standing at `length`, then read them again at exactly
/// their number. A commit that gives one more body that length cannot
/// observe the second selection again, so none of its facts is rebased. A
/// commit that leaves their number alone rebases.
pub(crate) fn survey_standing_length<Schema, Binding>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    length: worth_query_consumer_values::PositiveLength,
) -> Result<(), HandlerExecutionDenial>
where
    Schema: TopologySchemaBinding,
    Binding: worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>,
    Length: OperationReads<Binding::Operation>,
{
    let standing = reader.select_entities(Length::reference(), length, SURVEY_WIDTH)?;
    reader
        .select_entities(Length::reference(), length, standing.len())
        .map(drop)
}

#[cfg(test)]
static DECISION_ENTRIES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Counts actual handler entries; source-input preparation does not enter here.
#[cfg(test)]
pub(crate) fn take_decision_entries() -> usize {
    DECISION_ENTRIES.swap(0, std::sync::atomic::Ordering::SeqCst)
}

pub struct PlanarHandler;
trait PlanarHandlerBinding<Schema: TopologySchemaBinding>:
    worth_query_decl::facade::application_operation::ApplicationMutationBinding<
    Schema,
    Input = PlanarMutation,
    Decision = (),
    Result = PlanarAdjustmentResult,
    Denial = PlanarMutationDenial,
    Output = PlanarOutputs,
>
{
}

impl<Schema: TopologySchemaBinding> PlanarHandlerBinding<Schema> for PlanarMutationBinding<Schema> {}
impl<Schema: TopologySchemaBinding> PlanarHandlerBinding<Schema> for PlanarEditBinding<Schema> {}

impl<Schema: TopologySchemaBinding, Binding: PlanarHandlerBinding<Schema>>
    OperationHandler<Schema, Binding> for PlanarHandler
where
    Body: OperationReads<Binding::Operation> + OperationCreates<Binding::Operation>,
    BodyKey: OperationReads<Binding::Operation> + OperationWrites<Binding::Operation>,
    PositionX: OperationReads<Binding::Operation> + OperationWrites<Binding::Operation>,
    PositionY: OperationReads<Binding::Operation> + OperationWrites<Binding::Operation>,
    Length: OperationReads<Binding::Operation> + OperationWrites<Binding::Operation>,
    PlanarSuccessor: OperationReads<Binding::Operation>
        + OperationLinks<Binding::Operation>
        + OperationUnlinks<Binding::Operation>,
{
    fn decide(
        &self,
        input: &PlanarMutation,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<(), PlanarMutationDenial> {
        #[cfg(feature = "test-query-execution-observer")]
        HANDLER_CONTACTS.with(|contacts| contacts.set(contacts.get() + 1));
        #[cfg(test)]
        DECISION_ENTRIES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
            PlanarOperation::PublishDerivedOutput(output) => {
                // These fixture keys exercise actual context consumption for
                // the opted-in initial producer. The ordinary key keeps the
                // tracked projection-only decision path.
                if output.body_key == "anchor-isolated" {
                    let _ = reader.reader().version();
                } else if output.body_key == "anchor-island" {
                    let _ = reader.key_identity();
                } else if output.body_key.starts_with(SURVEYED_BODY_KEY_PREFIX) {
                    if let Err(error) = survey_standing_length(reader, output.value) {
                        return HandlerResult::ExecutionDenied(error);
                    }
                }
                let entity =
                    match reader.resolve_entity(BodyKey::reference(), output.body_key.clone()) {
                        Ok(entity) => entity,
                        Err(error) => return HandlerResult::ExecutionDenied(error),
                    };
                match reader.field(&entity, Length::reference()) {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        return HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate)
                    }
                    Err(error) => return HandlerResult::ExecutionDenied(error),
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
            PlanarOperation::VerifyCurrentOutputs(expectations)
            | PlanarOperation::VerifyFinalCurrentOutputs(expectations) => {
                for expectation in expectations {
                    let producer = match reader
                        .resolve_entity(BodyKey::reference(), expectation.producer_key.clone())
                    {
                        Ok(entity) => entity,
                        Err(error) => return HandlerResult::ExecutionDenied(error),
                    };
                    let selected = match &input.operation {
                        PlanarOperation::VerifyFinalCurrentOutputs(_) => {
                            reader.current_output::<PlanarFinalOutputFamily, Body>(&producer)
                        }
                        _ => reader.current_output::<PlanarOutputFamily, Body>(&producer),
                    };
                    let output = match selected {
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
            PlanarOperation::PublishDerivedOutput(_) => (0, 0, 0, 1),
            PlanarOperation::RetargetSuccessor { .. } => (0, 1, 1, 0),
            PlanarOperation::VerifyCurrentOutputs(_)
            | PlanarOperation::VerifyFinalCurrentOutputs(_) => (0, 0, 0, 0),
        };
        requirements(creates, links, unlinks, writes, 8192)
    }
    fn build_candidate(
        &self,
        input: &PlanarMutation,
        _: (),
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        match author(input, writer) {
            Ok(changed_vertices) => {
                HandlerResult::Completed(PlanarAdjustmentResult { changed_vertices })
            }
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
fn author<Schema: TopologySchemaBinding, Binding: PlanarHandlerBinding<Schema>>(
    input: &PlanarMutation,
    writer: &mut CandidateWriter<'_, Schema, Binding>,
) -> Result<usize, HandlerExecutionDenial>
where
    Body: OperationReads<Binding::Operation> + OperationCreates<Binding::Operation>,
    BodyKey: OperationReads<Binding::Operation> + OperationWrites<Binding::Operation>,
    PositionX: OperationReads<Binding::Operation> + OperationWrites<Binding::Operation>,
    PositionY: OperationReads<Binding::Operation> + OperationWrites<Binding::Operation>,
    Length: OperationReads<Binding::Operation> + OperationWrites<Binding::Operation>,
    PlanarSuccessor: OperationReads<Binding::Operation>
        + OperationLinks<Binding::Operation>
        + OperationUnlinks<Binding::Operation>,
{
    let anchor = writer
        .resolve_entity(BodyKey::reference(), input.scope_key.clone())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .preserve_output::<PlanarAnchorOutput<Schema>>(&anchor)
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
                writer
                    .create_member::<PlanarCreatedOutputs<Schema>>(&vertex.body_key, &entity)
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
        PlanarOperation::PublishDerivedOutput(output) => {
            let entity = writer
                .resolve_entity(BodyKey::reference(), output.body_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&entity, Length::reference(), output.value)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(1)
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
        PlanarOperation::VerifyCurrentOutputs(_)
        | PlanarOperation::VerifyFinalCurrentOutputs(_) => Ok(0),
    }
}

#[cfg(feature = "test-query-execution-observer")]
thread_local! { static HANDLER_CONTACTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(all(test, feature = "test-query-execution-observer"))]
pub(crate) fn handler_contacts() -> usize {
    HANDLER_CONTACTS.with(std::cell::Cell::get)
}
