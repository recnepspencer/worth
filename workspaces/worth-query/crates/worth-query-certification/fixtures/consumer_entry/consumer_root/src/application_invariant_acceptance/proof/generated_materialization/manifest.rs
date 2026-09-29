use worth_query_consumer_values::PlanarVertex;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOptionalOutputRole, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput, WorthQueryGeneratedEntity, WorthQueryGeneratedOutputReconstruction,
    WorthQueryGeneratedOutputReconstructionDenial,
};
use worth_query_topology_entry::{
    final_auxiliary_output, final_closing_output, Body, BodyKey, FinalPlanarMutationBinding,
    Length, PlanarFinalOutputProducer, PositionX, PositionY,
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
    if index == 2 {
        return reconstruction
            .entity(final_closing_output::<ConsumerSchema>(), Body::reference())
            .expect("the optional closing role resolves its retained platform identity")
            .expect("the suspended output bound its closing vertex");
    }
    reconstruction
        .entity(role(vertex, index), Body::reference())
        .expect("the typed role resolves its retained platform identity")
}

/// An unbound at-most-one role reads as `None` and claims nothing, and a token
/// whose cardinality differs from the declaration is refused either way.
pub(super) fn require_optional_role_reads(
    reconstruction: &mut WorthQueryGeneratedOutputReconstruction<
        '_,
        ConsumerSchema,
        PlanarFinalOutputProducer<ConsumerSchema>,
    >,
) {
    assert!(reconstruction
        .entity(
            final_auxiliary_output::<ConsumerSchema>(),
            Body::reference()
        )
        .expect("an absent optional role is a value, not a denial")
        .is_none());
    let closing_as_required = WorthQueryApplicationOutputRole::<
        FinalPlanarMutationBinding<ConsumerSchema>,
        Body,
        WorthQueryCreateOutput,
    >::from_static("closing");
    let anchor_as_optional = WorthQueryApplicationOptionalOutputRole::<
        FinalPlanarMutationBinding<ConsumerSchema>,
        Body,
        WorthQueryCreateOutput,
    >::from_static("anchor");
    assert_eq!(
        reconstruction
            .entity(closing_as_required, Body::reference())
            .err(),
        Some(WorthQueryGeneratedOutputReconstructionDenial::OutputRoleCardinalityMismatch)
    );
    assert_eq!(
        reconstruction
            .entity(anchor_as_optional, Body::reference())
            .err(),
        Some(WorthQueryGeneratedOutputReconstructionDenial::OutputRoleCardinalityMismatch)
    );
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
