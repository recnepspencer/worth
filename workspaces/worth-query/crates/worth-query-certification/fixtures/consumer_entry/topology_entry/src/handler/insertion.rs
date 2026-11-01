//! A genuine atomic B->C to B->D->C contour edit with tracked old correspondence.
use super::*;
use worth_query_consumer_values::PlanarVertex;

pub(super) fn decide<S: TopologySchemaBinding, B: PlanarHandlerBinding<S>>(
    reader: &mut DecisionReader<'_, '_, '_, S, B>,
    after_key: &str,
    previous_next_key: &str,
    vertex: &PlanarVertex,
) -> HandlerResult<(), PlanarMutationDenial>
where
    Body: OperationReads<B::Operation>,
    BodyKey: OperationReads<B::Operation>,
    PlanarSuccessor: OperationReads<B::Operation>,
{
    let observed = (|| {
        let after = reader.resolve_entity(BodyKey::reference(), after_key.to_owned())?;
        let previous = reader.resolve_entity(BodyKey::reference(), previous_next_key.to_owned())?;
        let next = reader
            .related_one(PlanarSuccessor::reference(), &after)
            .map_err(HandlerExecutionDenial::new)?;
        let incoming = reader
            .related_one_incoming(PlanarSuccessor::reference(), &previous)
            .map_err(HandlerExecutionDenial::new)?;
        if next != previous || incoming != after {
            return Ok(Some(PlanarMutationDenial::UnexpectedSuccessor));
        }
        if !reader
            .select_entities(BodyKey::reference(), vertex.body_key.clone(), 2)?
            .is_empty()
        {
            return Ok(Some(PlanarMutationDenial::VertexKeyTaken));
        }
        Ok(None)
    })();
    match observed {
        Ok(None) => HandlerResult::Completed(()),
        Ok(Some(denial)) => HandlerResult::DomainDenied(denial),
        Err(cause) => HandlerResult::ExecutionDenied(cause),
    }
}

pub(super) fn insert<S: TopologySchemaBinding, B: PlanarHandlerBinding<S>>(
    writer: &mut CandidateWriter<'_, S, B>,
    after_key: &str,
    previous_next_key: &str,
    vertex: &PlanarVertex,
) -> Result<usize, HandlerExecutionDenial>
where
    Body: OperationCreates<B::Operation> + OperationReads<B::Operation>,
    BodyKey: OperationReads<B::Operation> + OperationWrites<B::Operation>,
    PositionX: OperationWrites<B::Operation>,
    PositionY: OperationWrites<B::Operation>,
    Length: OperationWrites<B::Operation>,
    PlanarSuccessor: OperationReads<B::Operation>
        + OperationLinks<B::Operation>
        + OperationUnlinks<B::Operation>,
{
    let after = writer
        .resolve_entity(BodyKey::reference(), after_key.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    let previous = writer
        .resolve_entity(BodyKey::reference(), previous_next_key.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    let key = WorthQueryApplicationEntityKey::new(&vertex.body_key)
        .map_err(HandlerExecutionDenial::new)?;
    let added = writer
        .create_entity_in_context(&after, Body::reference(), key)
        .map_err(HandlerExecutionDenial::new)?;
    crate::planar_vertex::initialize_planar_vertex(writer, &added, vertex)?;
    writer
        .create_member::<PlanarCreatedOutputs<S>>(&vertex.body_key, &added)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .unlink(PlanarSuccessor::reference(), &after, &previous)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .link(
            PlanarSuccessor::reference(),
            format!("insert:{}:incoming", vertex.body_key),
            &after,
            &added,
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .link(
            PlanarSuccessor::reference(),
            format!("insert:{}:outgoing", vertex.body_key),
            &added,
            &previous,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(1)
}
