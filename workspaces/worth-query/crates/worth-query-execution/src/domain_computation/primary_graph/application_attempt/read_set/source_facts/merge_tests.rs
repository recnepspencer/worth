use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, VersionId},
    runtime::RelationalAdjacencyDirection,
};

use super::{merge_source_facts, WorthQueryApplicationObservedFact as Fact};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind as Kind;

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

fn adjacency(revision: u64, work: usize, endpoints: Vec<EntityId>) -> Fact {
    Fact::SourceAdjacencyRevision {
        relation_kind: KindId::new(91),
        anchor: entity(1),
        direction: RelationalAdjacencyDirection::Outgoing,
        native_revision: Some(VersionId(revision)),
        comparison_work_limit: work,
        endpoints,
    }
}

#[test]
fn admitted_and_dependent_same_revision_adjacencies_compose_coverage() {
    let admitted = adjacency(13, 1, vec![entity(2)]);
    let dependent = adjacency(13, 4, vec![entity(3), entity(2)]);
    let expected = vec![adjacency(13, 4, vec![entity(2), entity(3)])];
    assert_eq!(
        merge_source_facts(
            vec![admitted.clone()],
            vec![dependent.clone()],
            "PreserveSplit"
        )
        .unwrap(),
        expected
    );
    assert_eq!(
        merge_source_facts(vec![dependent], vec![admitted], "PreserveSplit").unwrap(),
        expected
    );
}

#[test]
fn admitted_and_dependent_different_native_revisions_remain_denied() {
    let denial = merge_source_facts(
        vec![adjacency(13, 1, vec![entity(2)])],
        vec![adjacency(14, 4, vec![entity(3)])],
        "PreserveSplit",
    )
    .unwrap_err();
    assert_eq!(denial.kind(), Kind::DecisionDependencyMismatch);
    assert!(denial
        .subject()
        .contains("PreserveSplit: source facts conflict at"));
}

#[test]
fn different_native_locators_remain_separate_dependencies() {
    let first = adjacency(13, 1, vec![entity(2)]);
    let mut second = adjacency(13, 4, vec![entity(3)]);
    if let Fact::SourceAdjacencyRevision { anchor, .. } = &mut second {
        *anchor = entity(4);
    }
    let merged =
        merge_source_facts(vec![first.clone()], vec![second.clone()], "PreserveSplit").unwrap();
    assert_eq!(merged.len(), 2);
    assert!(merged.contains(&first));
    assert!(merged.contains(&second));
}
