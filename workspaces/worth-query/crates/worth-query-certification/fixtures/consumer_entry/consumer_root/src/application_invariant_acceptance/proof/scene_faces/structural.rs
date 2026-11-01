//! Real invalid installation seed transactions reach Native cardinality laws.
use super::*;
use crate::application_invariant_acceptance::resources;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_host::facade::application_invariants::RelationCardinalityBoundary;
use worth_query_host::facade::{application_installation::*, primary_graph::*};
use worth_query_topology_entry::{
    Body, BodyKey, Face, FaceAnchor, FaceKey, Length, PlanarSuccessor, PositionX, PositionY, Scene,
    SceneFace, SceneKey, TopologyConfiguration,
};

#[test]
fn missing_scene_membership_and_missing_anchor_refuse_real_seed_publication() {
    for missing_anchor in [false, true] {
        let result = in_memory_program(
            crate::application_program::validated_program().unwrap(),
            ConsumerSchema::declaration().unwrap(),
            (
                TopologyConfiguration {
                    final_feature: worth_query_topology_entry::planar_final_feature_runtime(),
                    setup_calls: Arc::new(AtomicUsize::new(0)),
                    invariant_calls: Arc::new(AtomicUsize::new(0)),
                    invariant_probe: Arc::new(AtomicUsize::new(0)),
                    invariant_read_observer: None,
                    producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
                    producer_domain_denial: Arc::new(AtomicBool::new(false)),
                },
                Arc::new(AtomicUsize::new(0)),
                (),
                worth_query_topology_entry::SceneFaceConfiguration::default(),
            ),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            resources::application_limits(),
            |phase, graph, _installed| {
                for (key, x, y) in [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)] {
                    graph.bind_entity(
                        WorthQueryApplicationEntitySeed::new(Body::reference(), entity_key(key))
                            .field(BodyKey::reference(), key.to_owned())
                            .field(Length::reference(), length(1))
                            .field(PositionX::reference(), length(x))
                            .field(PositionY::reference(), length(y)),
                    )?;
                }
                for (a, b) in [("a", "b"), ("b", "c"), ("c", "a")] {
                    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                        PlanarSuccessor::reference(),
                        format!("{a}-{b}"),
                        entity_key(a),
                        entity_key(b),
                    ))?;
                }
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Scene::reference(), entity_key("scene"))
                        .field(SceneKey::reference(), "scene".to_owned()),
                )?;
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Face::reference(), entity_key("face"))
                        .field(FaceKey::reference(), "face".to_owned()),
                )?;
                if missing_anchor {
                    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                        SceneFace::reference(),
                        "membership",
                        entity_key("scene"),
                        entity_key("face"),
                    ))?;
                } else {
                    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                        FaceAnchor::reference(),
                        "anchor",
                        entity_key("face"),
                        entity_key("a"),
                    ))?;
                }
                graph.commit_seed_batch(phase)
            },
        );
        let Err(WorthQueryInMemoryApplicationDenial::InitialState(denial)) = result else {
            panic!("orphan seed must fail during real Native initial-state publication")
        };
        let Some(WorthQueryRelationLawDenial::Cardinality {
            count,
            limit,
            boundary,
            ..
        }) = denial.relation_law()
        else {
            panic!("{denial:?}")
        };
        assert_eq!((count, limit), (0, 1));
        assert_eq!(
            boundary,
            if missing_anchor {
                RelationCardinalityBoundary::Source
            } else {
                RelationCardinalityBoundary::Target
            }
        );
        assert!(denial.custom_invariant_denial().is_none());
    }
}
fn entity_key<E>(key: &str) -> WorthQueryApplicationEntityKey<ConsumerSchema, E> {
    WorthQueryApplicationEntityKey::new(key).unwrap()
}
