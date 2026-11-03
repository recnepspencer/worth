use std::sync::Arc;
use worth_foundational::facade::CanonicalDigestId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::identity::VersionId;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_facts::revisions::WorthQueryObservedAdjacencyRevision;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::root_selection::WorthQueryObservedRootSelection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSourceFootprint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection;
pub fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}
pub fn footprint(endpoints: Vec<EntityId>, revision: u64) -> WorthQueryObservedSourceFootprint {
    let adjacency = WorthQueryObservedAdjacencyRevision {
        anchor: entity(1),
        relation_kind: KindId(7),
        direction: worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing,
        native_revision: Some(VersionId(revision)),
        comparison_work_limit: 8,
        endpoints,
    };
    WorthQueryObservedSourceFootprint {
        root: entity(1),
        complete: true,
        entities: vec![entity(1)],
        aspects: Vec::new(),
        root_selection: Some(Arc::new(WorthQueryObservedRootSelection::new(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_facts::SourceFacts {
                entities: vec![entity(1)],
                aspects: Vec::new(),
                adjacencies: vec![adjacency.clone()],
                walks: None,
                derived: None,
                allocation: None,
            },
            None,
        ))),
        adjacencies: vec![adjacency],
    }
}
pub trait TestsWorthQueryObservedSourceSelectionForTest: Sized {
    fn selecting_for_test(endpoint: u64) -> Self;
}
impl TestsWorthQueryObservedSourceSelectionForTest for WorthQueryObservedSourceSelection {
    fn selecting_for_test(endpoint: u64) -> Self {
        let authority = worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test();
        let meaning = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry::new(authority)
            .intern(
                &[7; 32],
                &[8; 32],
                footprint(vec![entity(endpoint)], 4),
                &worth_query_execution::facade::domain_computation::WorthQueryApplicationBasisSelectionIdentity::Relational,
            )
            .unwrap();
        Self {
            runtime_authority: authority.as_u64(),
            schema_binding: ApplicationSchemaBindingIdentity::from_installed_parts(
                1,
                2,
                CanonicalDigestId::new([3; 32]),
                CanonicalDigestId::new([4; 32]),
            ),
            meaning,
            managed: None,
        }
    }
}
