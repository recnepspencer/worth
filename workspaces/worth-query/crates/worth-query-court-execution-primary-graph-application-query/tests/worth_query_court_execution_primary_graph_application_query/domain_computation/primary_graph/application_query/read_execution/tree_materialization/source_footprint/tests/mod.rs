use worth_foundational::facade::AspectContractRevision;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::FieldKey;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::runtime::RelationalAdjacencyDirection;
use worth_relational::facade::runtime::RelationalFieldPresence;
use worth_relational::facade::runtime::RelationalFieldRevision;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::tree_materialization::source_footprint::normalize_source_footprint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_facts::revisions::WorthQueryObservedAdjacencyRevision;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_facts::revisions::WorthQueryObservedFieldRevision;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSourceFootprint;

#[test]
fn normalization_deduplicates_revisited_native_dependencies() {
    let entity = EntityId::new(PartitionId::main(), 7, 1);
    let aspect = WorthQueryObservedFieldRevision {
        entity,
        entity_name: "Body".to_owned(),
        aspect: AspectKey::new("body-facts").unwrap(),
        field: FieldKey::new("body-facts").unwrap(),
        contract_revision: AspectContractRevision(1),
        native_revision: Some(RelationalFieldRevision::new(
            VersionId(11),
            RelationalFieldPresence::Present,
        )),
    };
    let incoming = adjacency(entity, RelationalAdjacencyDirection::Incoming);
    let outgoing = adjacency(entity, RelationalAdjacencyDirection::Outgoing);
    let mut footprint = WorthQueryObservedSourceFootprint {
        root: entity,
        complete: true,
        entities: vec![entity, entity],
        aspects: vec![aspect.clone(), aspect],
        adjacencies: vec![incoming.clone(), outgoing.clone(), incoming, outgoing],
        root_selection: None,
    };

    normalize_source_footprint(&mut footprint);

    assert_eq!((*footprint.entities_for_test()), vec![entity]);
    assert_eq!((*footprint.aspects_for_test()).len(), 1);
    assert_eq!(footprint.adjacencies.len(), 2);
    managed_order_matches_legacy_for_stable_keys_and_full_equality(&footprint);
}

pub fn adjacency(
    anchor: EntityId,
    direction: RelationalAdjacencyDirection,
) -> WorthQueryObservedAdjacencyRevision {
    WorthQueryObservedAdjacencyRevision {
        anchor,
        relation_kind: KindId::new(19),
        direction,
        native_revision: Some(VersionId(23)),
        comparison_work_limit: 4,
        endpoints: vec![EntityId::new(PartitionId::main(), 8, 1)],
    }
}

// Descriptive canonicalization data, not an observation or authority fixture.
pub fn managed_order_matches_legacy_for_stable_keys_and_full_equality(
    base: &WorthQueryObservedSourceFootprint,
) {
    use std::num::NonZeroUsize;
    use worth_execution::CancellationToken;
    use worth_execution::ExecutionRequest;
    use worth_execution::SerialRequest;
    use worth_foundational::DeterminismContract;
    use worth_foundational::ExecutionBudget;
    use worth_foundational::ExecutionPosture;
    use worth_foundational::ExecutionRequestPolicy;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_facts::allocation::SourceFactsAllocation;
    let backing = SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, None),
        ),
        CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&backing);
    let grant = SourceFactsAllocation::open(&execution).unwrap();
    let storage = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::tree_materialization::source_footprint::material_storage::MaterialFootprintStorage::new_for_test(&execution, Some(&grant));
    let mut earlier = (*base.aspects_for_test())[0].clone();
    earlier.entity_name = "first-stable-value".to_owned();
    let mut later = earlier.clone();
    later.entity_name = "second-stable-value".to_owned();
    later.native_revision = Some(RelationalFieldRevision::new(
        VersionId(12),
        RelationalFieldPresence::Present,
    ));
    let mut other = earlier.clone();
    other.field = FieldKey::new("other-field").unwrap();
    let a = adjacency((*base.root_for_test()), RelationalAdjacencyDirection::Incoming);
    let mut b = a.clone();
    (*b.endpoints_for_test()).push(EntityId::new(PartitionId::main(), 9, 2));
    for rotation in 0..5 {
        let mut input = WorthQueryObservedSourceFootprint {
            root: (*base.root_for_test()),
            complete: true,
            entities: vec![
                EntityId::new(PartitionId::main(), 8, 1),
                (*base.root_for_test()),
                (*base.root_for_test()),
            ],
            aspects: vec![
                other.clone(),
                later.clone(),
                earlier.clone(),
                later.clone(),
                earlier.clone(),
            ],
            adjacencies: vec![b.clone(), b.clone(), a.clone(), b.clone(), a.clone()],
            root_selection: None,
        };
        (*input.aspects_for_test()).rotate_left(rotation);
        input.adjacencies.rotate_right(rotation);
        let mut expected = input.clone();
        normalize_source_footprint(&mut expected);
        storage.normalize_for_test(&mut input).unwrap();
        assert_eq!(
            input, expected,
            "stable sort keys and full-value dedup differ at {rotation}"
        );
    }
    let mut empty = WorthQueryObservedSourceFootprint {
        root: (*base.root_for_test()),
        complete: true,
        entities: Vec::new(),
        aspects: Vec::new(),
        adjacencies: Vec::new(),
        root_selection: None,
    };
    storage.normalize_for_test(&mut empty).unwrap();
    assert!((*empty.entities_for_test()).is_empty() && (*empty.aspects_for_test()).is_empty() && empty.adjacencies.is_empty());
}
