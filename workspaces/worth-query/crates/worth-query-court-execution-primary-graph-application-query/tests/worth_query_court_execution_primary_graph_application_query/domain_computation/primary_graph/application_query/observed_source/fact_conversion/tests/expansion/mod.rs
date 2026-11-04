//! Focused physical-fact merging; literals describe coordinates, never authority.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::fact_conversion::materialization::merge_facts;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::fact_conversion::control::SourceFactExpansionControl;
use crate::domain_computation::primary_graph::application_query::observed_source::fact_conversion::tests::*;
use worth_execution::CancellationSource;
use worth_execution::ExecutionRequest;
use worth_execution::MapKernelStop;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;
use worth_execution::WorkCeilingDenial;
pub fn entity(generation: u32) -> EntityId {
    EntityId::new(PartitionId::main(), 12, generation)
}
pub fn adjacency(revision: u64, endpoints: &[EntityId]) -> Fact {
    Fact::SourceAdjacencyRevision {
        anchor: entity(1), relation_kind: KindId::new(9), direction: RelationalAdjacencyDirection::Outgoing,
        native_revision: Some(VersionId(revision)), comparison_work_limit: 1,
        endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(endpoints, worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
    }
}
pub fn field(revision: u64) -> Fact {
    use worth_foundational::facade::AspectFieldLocator;
    use worth_foundational::facade::AspectKey;
    use worth_foundational::facade::CanonicalFieldPath;
    use worth_foundational::facade::FieldKey;
    use worth_foundational::facade::LocatorAuthority;
    Fact::SourceFieldRevision {
        entity_id: entity(1),
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            AspectKey::new("Facts").unwrap(),
            CanonicalFieldPath::single(FieldKey::new("value").unwrap()),
        ),
        native_revision: Some(
            worth_relational::facade::runtime::RelationalFieldRevision::new(
                VersionId(revision),
                worth_relational::facade::runtime::RelationalFieldPresence::Present,
            ),
        ),
    }
}
#[test]
fn request_bound_fact_merge_keeps_generations_and_merges_complete_endpoint_bodies() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let control =
        SourceFactExpansionControl::in_request_for_test(ExecutionRequest::serial(&request), None);
    let facts = merge_facts(
        vec![
            Fact::SourceEntity {
                entity_id: entity(1),
            },
            Fact::SourceEntity {
                entity_id: entity(2),
            },
            Fact::SourceEntity {
                entity_id: entity(1),
            },
            adjacency(7, &[entity(2)]),
            adjacency(7, &[entity(1)]),
        ],
        control,
        "merge",
    )
    .unwrap();
    assert_eq!(facts.len(), 3);
    assert!(matches!(facts[0], Fact::SourceEntity { entity_id } if entity_id == entity(1)));
    assert!(matches!(facts[1], Fact::SourceEntity { entity_id } if entity_id == entity(2)));
    assert!(
        matches!(&facts[2], Fact::SourceAdjacencyRevision { endpoints, .. } if endpoints.as_ref() == [entity(1), entity(2)])
    );
    let repeated = merge_facts(vec![field(7), field(7)], control, "field twin").unwrap();
    assert_eq!(repeated.len(), 1);
    let field_conflict =
        merge_facts(vec![field(7), field(8)], control, "field conflict").unwrap_err();
    assert_eq!(
        field_conflict.kind(),
        worth_query_execution::facade::domain_computation::WorthQuerySourceExpectationDenialKind::SourceChanged
    );
    let conflict = merge_facts(
        vec![adjacency(7, &[]), adjacency(8, &[])],
        control,
        "conflict",
    )
    .unwrap_err();
    assert_eq!(
        conflict.kind(),
        worth_query_execution::facade::domain_computation::WorthQuerySourceExpectationDenialKind::SourceChanged
    );
}
#[test]
fn direct_fact_retention_preserves_original_execution_cancellation() {
    let stop = CancellationSource::new();
    let request = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let control =
        SourceFactExpansionControl::in_request_for_test(ExecutionRequest::serial(&request), None);
    stop.cancel();
    let refused = merge_facts(
        vec![Fact::SourceEntity {
            entity_id: entity(1),
        }],
        control,
        "cancelled",
    )
    .unwrap_err();
    assert_eq!(
        refused.execution_denial(),
        Some(&WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    );
    let fresh = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    assert!(merge_facts(
        vec![Fact::SourceEntity {
            entity_id: entity(1)
        }],
        SourceFactExpansionControl::in_request_for_test(ExecutionRequest::serial(&fresh), None),
        "fresh"
    )
    .is_ok());
}
