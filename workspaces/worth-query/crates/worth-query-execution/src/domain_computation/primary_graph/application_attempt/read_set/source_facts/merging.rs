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
    let merged = merge_source_facts(vec![admitted.clone()], keyed([dependent.clone()]), "test")
        .unwrap()
        .into_values()
        .collect::<Vec<_>>();
    assert_eq!(merged, vec![adjacency(5, 64, vec![first, second])]);
    assert_eq!(
        merge_source_facts(vec![dependent], keyed([admitted]), "test")
            .unwrap()
            .into_values()
            .collect::<Vec<_>>(),
        merged,
    );
}

#[test]
fn admitted_and_dependent_reads_reject_different_native_revisions() {
    let error = merge_source_facts(
        vec![adjacency(5, 2, vec![])],
        keyed([adjacency(6, 64, vec![])]),
        "test",
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch
    );
}

fn keyed(
    facts: impl IntoIterator<Item = WorthQueryApplicationObservedFact>,
) -> BTreeMap<WorthQueryApplicationFactStorageKey, WorthQueryApplicationObservedFact> {
    facts
        .into_iter()
        .map(|fact| (fact.dependency_key(), fact))
        .collect()
}

#[test]
fn moving_dependent_source_map_keeps_its_backing_without_an_intermediate_vector() {
    let filter = concat!(module_path!(), "::isolated_source_map_transfer")
        .split_once("::")
        .unwrap()
        .1;
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", filter, "--test-threads=1", "--nocapture"])
        .env("WORTH_QUERY_SOURCE_MAP_TRANSFER_PROBE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "source-map allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
}

#[test]
fn isolated_source_map_transfer() {
    if std::env::var_os("WORTH_QUERY_SOURCE_MAP_TRANSFER_PROBE").is_none() {
        return;
    }
    // This pure storage handoff grants no graph observation authority. Prepare
    // its keyed container before measuring; transfer creates no new backing.
    let dependent = keyed(
        (1..=128).map(|slot| WorthQueryApplicationObservedFact::SourceEntity {
            entity_id: EntityId::new(PartitionId::main(), slot, 1),
        }),
    );
    let count = dependent.len();
    let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    let merged = merge_source_facts(Vec::new(), dependent, "test").unwrap();
    let allocated = region.change().bytes_allocated;
    assert_eq!(merged.len(), count);
    assert_eq!(
        allocated, 0,
        "the transferred map must remain owned until final admitted array emission"
    );
}

#[test]
fn source_storage_keeps_ordinary_order_and_projected_canonical_order() {
    let first = EntityId::new(PartitionId::main(), 1, 1);
    let second = EntityId::new(PartitionId::main(), 2, 1);
    let fact = |entity_id| WorthQueryApplicationObservedFact::SourceEntity { entity_id };
    let ordinary = SourceFacts::Admitted(vec![fact(second), fact(first)]);
    assert_eq!(
        ordinary.into_values().collect::<Vec<_>>(),
        vec![fact(second), fact(first)]
    );
    let projected = merge_source_facts(vec![fact(second)], keyed([fact(first)]), "test").unwrap();
    assert_eq!(
        projected.into_values().collect::<Vec<_>>(),
        vec![fact(first), fact(second)]
    );
}
