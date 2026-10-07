use super::*;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, VersionId},
    runtime::RelationalAdjacencyDirection,
};

fn adjacency(
    revision: u64,
    limit: usize,
    endpoints: Vec<EntityId>,
) -> WorthQueryApplicationObservedFact {
    WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind: KindId::new(3),
        anchor: EntityId::new(PartitionId::main(), 1, 1),
        direction: RelationalAdjacencyDirection::Outgoing,
        native_revision: Some(VersionId(revision)),
        comparison_work_limit: limit,
        endpoints,
    }
}

#[test]
fn admitted_and_dependent_reads_merge_one_native_revision_with_complete_coverage() {
    let first = EntityId::new(PartitionId::main(), 2, 1);
    let second = EntityId::new(PartitionId::main(), 4, 1);
    let admitted = adjacency(5, 2, vec![second, first]);
    let dependent = adjacency(5, 64, vec![first]);
    let merged =
        merge_source_facts(vec![admitted.clone()], vec![dependent.clone()], "test").unwrap();
    assert_eq!(merged, vec![adjacency(5, 64, vec![first, second])]);
    assert_eq!(
        merge_source_facts(vec![dependent], vec![admitted], "test").unwrap(),
        merged,
    );
}

#[test]
fn admitted_and_dependent_reads_reject_different_native_revisions() {
    let error = merge_source_facts(
        vec![adjacency(5, 2, vec![])],
        vec![adjacency(6, 64, vec![])],
        "test",
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch
    );
}
