//! One initial field contract for a caller-allocated actual planar Body.
use super::*;
use worth_query_consumer_values::PlanarVertex;
use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{OperationCreates, OperationWrites},
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, HandlerExecutionDenial, WorthQueryApplicationEffectEntity,
};

pub(crate) fn initialize_planar_vertex<S: TopologySchemaBinding, B: ApplicationMutationBinding<S>>(
    writer: &mut CandidateWriter<'_, S, B>,
    entity: &WorthQueryApplicationEffectEntity<S, Body>,
    vertex: &PlanarVertex,
) -> Result<(), HandlerExecutionDenial>
where
    Body: OperationCreates<B::Operation>,
    BodyKey: OperationWrites<B::Operation>,
    PositionX: OperationWrites<B::Operation>,
    PositionY: OperationWrites<B::Operation>,
    Length: OperationWrites<B::Operation>,
{
    writer
        .initialize_field(entity, BodyKey::reference(), vertex.body_key.clone())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(entity, PositionX::reference(), vertex.x)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(entity, PositionY::reference(), vertex.y)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(
            entity,
            Length::reference(),
            PositiveLength::new(1).expect("positive initial length"),
        )
        .map_err(HandlerExecutionDenial::new)
}
