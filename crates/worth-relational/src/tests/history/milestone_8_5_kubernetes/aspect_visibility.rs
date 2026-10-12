use super::*;

pub(super) fn replicas_canonical_bytes(
    record: &crate::storage::data::EntityReadRecord,
) -> Option<Vec<u8>> {
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("replicas").expect("valid replicas aspect"),
        CanonicalFieldPath::single(FieldKey::new("replicas").expect("valid replicas field")),
    );
    crate::visibility::materialization::read_records::entity_query_locus_comparison_key(
        record, &locator,
    )
    .map(|key| key.canonical_value_bytes().to_vec())
}

pub(super) fn visible_truth_for_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    entity: crate::facade::identity::EntityId,
) -> KubernetesVisibleTruthEvidence {
    let snapshot = crate::tests::support::snapshot_for_owner_branch(runtime, branch_id);
    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("owner-admitted branch snapshot remains readable");
    let record = read
        .get_entity(entity)
        .expect("entity remains visible on certified branch");
    let evidence = KubernetesVisibleTruthEvidence {
        entity_name: read_entity_name(record),
        replicas_canonical_bytes: replicas_canonical_bytes(record),
    };
    drop(read);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&snapshot)
        .is_ok());
    evidence
}
