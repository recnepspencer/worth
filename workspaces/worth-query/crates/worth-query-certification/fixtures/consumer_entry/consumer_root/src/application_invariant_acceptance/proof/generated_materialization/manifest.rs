use worth_query_consumer_values::PlanarVertex;
use worth_query_host::facade::primary_graph::{
    WorthQueryGeneratedEntity, WorthQueryGeneratedOutputReconstruction,
};
use worth_query_topology_entry::{
    Body, BodyKey, Length, PlanarFinalOutputProducer, PositionX, PositionY,
};

use super::super::length;
use super::expected::role;
use crate::ConsumerSchema;

pub(super) fn claim_entity(
    reconstruction: &mut WorthQueryGeneratedOutputReconstruction<
        '_,
        ConsumerSchema,
        PlanarFinalOutputProducer<ConsumerSchema>,
    >,
    vertex: &PlanarVertex,
    index: usize,
) -> WorthQueryGeneratedEntity<ConsumerSchema, Body> {
    reconstruction
        .entity(role(vertex, index), Body::reference())
        .expect("the typed role resolves its retained platform identity")
}

pub(super) fn write_fields(
    reconstruction: &mut WorthQueryGeneratedOutputReconstruction<
        '_,
        ConsumerSchema,
        PlanarFinalOutputProducer<ConsumerSchema>,
    >,
    entity: &WorthQueryGeneratedEntity<ConsumerSchema, Body>,
    vertex: &PlanarVertex,
) {
    reconstruction
        .field(entity, BodyKey::reference(), vertex.body_key.clone())
        .expect("the typed body key encodes through Query");
    reconstruction
        .field(entity, PositionX::reference(), vertex.x)
        .expect("the typed x coordinate encodes through Query");
    reconstruction
        .field(entity, PositionY::reference(), vertex.y)
        .expect("the typed y coordinate encodes through Query");
    reconstruction
        .field(entity, Length::reference(), length(4))
        .expect("the typed length encodes through Query");
}
