use worth_query_consumer_values::PlanarVertex;
use worth_query_host::facade::primary_graph::{
    WorthQueryGeneratedEntity, WorthQueryGeneratedOutputReconstruction,
    WorthQueryGeneratedOutputReconstructionDenial, WorthQueryReconstructedOutputEntity,
};
use worth_query_topology_entry::{
    Body, BodyKey, FinalAnchorOutput, FinalAuxiliaryOutput, FinalClosingOutput,
    FinalCreatedOutputs, FinalRetainedSourceOutput, FinalRetainedSources, Length,
    PlanarFinalOutputProducer, PositionX, PositionY,
};

use super::super::length;
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
    match index {
        0 => reconstruction.output::<FinalAnchorOutput<ConsumerSchema>>(),
        1 => reconstruction.output_member::<FinalCreatedOutputs<ConsumerSchema>>(&vertex.body_key),
        _ => {
            return reconstruction
                .output::<FinalClosingOutput<ConsumerSchema>>()
                .expect("the optional closing role resolves its retained platform identity")
                .map(generated)
                .expect("the suspended output bound its closing vertex");
        }
    }
    .map(generated)
    .expect("the typed role resolves its retained platform identity")
}

fn generated(
    entity: WorthQueryReconstructedOutputEntity<ConsumerSchema, Body>,
) -> WorthQueryGeneratedEntity<ConsumerSchema, Body> {
    match entity {
        WorthQueryReconstructedOutputEntity::Generated(entity) => entity,
        WorthQueryReconstructedOutputEntity::Retained(_) => {
            panic!("created ring vertices must be in generated custody")
        }
    }
}

/// An unbound at-most-one role reads as `None` and claims nothing. Reading a
/// role with a cardinality other than its declaration fails to compile.
pub(super) fn require_optional_role_reads(
    reconstruction: &mut WorthQueryGeneratedOutputReconstruction<
        '_,
        ConsumerSchema,
        PlanarFinalOutputProducer<ConsumerSchema>,
    >,
) {
    assert!(reconstruction
        .entity::<FinalAuxiliaryOutput<ConsumerSchema>>()
        .expect("an absent optional role is a value, not a denial")
        .is_none());
    assert!(reconstruction
        .output::<FinalAuxiliaryOutput<ConsumerSchema>>()
        .unwrap()
        .is_none());
    assert!(matches!(
        reconstruction
            .output::<FinalRetainedSourceOutput<ConsumerSchema>>()
            .unwrap(),
        WorthQueryReconstructedOutputEntity::Retained(_)
    ));
    // Both selectors refer to the actual upstream source, which was not suspended.
    for _ in 0..2 {
        assert!(matches!(
            reconstruction
                .output_member::<FinalRetainedSources<ConsumerSchema>>("anchor-b")
                .unwrap(),
            WorthQueryReconstructedOutputEntity::Retained(_)
        ));
    }
    for suffix in ["missing-source", ""] {
        match reconstruction.output_member::<FinalRetainedSources<ConsumerSchema>>(suffix) {
            Err(denial) => assert_eq!(
                denial,
                WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole
            ),
            Ok(_) => panic!("an absent preserved member must not yield a retained handle"),
        }
    }
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
