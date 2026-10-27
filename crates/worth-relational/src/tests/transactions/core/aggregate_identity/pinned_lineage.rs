//! Real committed identities remain inspectable independently of current membership.
use super::fixtures::*;
use crate::facade::runtime::{
    PinnedAggregateLineageScope as Scope, PinnedAggregateReadDenialKind as Denial,
};
use crate::tests::support::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AggregateDefinitionIdentity, AggregateIdentity, AggregateLineageKind,
};

#[test]
fn losing_identity_resolves_two_real_merges_and_old_pins_keep_their_own_terminal() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[0, 1, 2]));
    let ids = members(&born, 3);
    let rows = records(execution, &runtime, &born.snapshot);
    let [c, a, b] = [
        identity(&rows, ids[0]),
        identity(&rows, ids[1]),
        identity(&rows, ids[2]),
    ];
    let original = runtime
        .read_truth()
        .pin_read_capability(&born.snapshot, &[], execution)
        .unwrap();
    create_branch_from_main(&runtime, "lineage-sibling");
    let sibling_snapshot = snapshot_for_owner_branch(&runtime, &BranchId("lineage-sibling".into()));
    let sibling = runtime
        .read_truth()
        .pin_read_capability(&sibling_snapshot, &[], execution)
        .unwrap();
    let first = commit(
        execution,
        &runtime,
        "main",
        rewire(
            execution,
            &runtime,
            &born.snapshot,
            &[(ids[1], ids[2]), (ids[2], ids[1])],
        ),
    );
    let first_pin = runtime
        .read_truth()
        .pin_read_capability(&first.snapshot, &[], execution)
        .unwrap();
    let second = commit(
        execution,
        &runtime,
        "main",
        rewire(
            execution,
            &runtime,
            &first.snapshot,
            &[(ids[0], ids[1]), (ids[2], ids[0])],
        ),
    );
    let latest = runtime
        .read_truth()
        .pin_read_capability(&second.snapshot, &[], execution)
        .unwrap();
    let scope = || Scope::new(KindId(2), descriptor().definition, b);
    let before = original.read_aggregate_lineage(execution, scope()).unwrap();
    assert_eq!(before.latest_identity(), Some(b));
    assert_eq!(before.retired_at(), None);
    assert_eq!(before.lineage().count(), 0);
    let middle = first_pin
        .read_aggregate_lineage(execution, scope())
        .unwrap();
    assert_eq!(middle.latest_identity(), Some(a));
    assert_eq!(middle.resolution_steps().count(), 1);
    let resolved = latest.read_aggregate_lineage(execution, scope()).unwrap();
    assert_eq!(resolved.definition(), descriptor().definition);
    assert_eq!(resolved.identity(), b);
    assert_eq!(resolved.retired_at(), Some(first.commit.commit_id));
    assert_eq!(resolved.latest_identity(), Some(c));
    assert_eq!(
        resolved
            .resolution_steps()
            .map(|event| (event.occurrence(), event.kind(), event.from(), event.to()))
            .collect::<Vec<_>>(),
        vec![
            (
                first.commit.commit_id,
                AggregateLineageKind::MergedInto,
                b,
                a
            ),
            (
                second.commit.commit_id,
                AggregateLineageKind::MergedInto,
                a,
                c
            ),
        ]
    );
    assert_eq!(
        resolved
            .lineage()
            .map(|event| (event.from(), event.to()))
            .collect::<Vec<_>>(),
        vec![(b, a)]
    );
    assert_eq!(
        sibling
            .read_aggregate_lineage(execution, scope())
            .unwrap()
            .latest_identity(),
        Some(b)
    );
    drop((before, middle, resolved));
    for result in [&born, &first, &second] {
        release_test_commit_snapshot(&runtime, result);
    }
    runtime
        .snapshots()
        .release_snapshot(&sibling_snapshot)
        .unwrap();
    assert_eq!(
        original
            .read_aggregate_lineage(execution, scope())
            .unwrap()
            .latest_identity(),
        Some(b)
    );
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn disappeared_members_leave_a_real_retired_terminal_distinct_from_missing_identity() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 0]));
    let ids = members(&born, 2);
    let identity = identity(&records(execution, &runtime, &born.snapshot), ids[0]);
    let read = runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .unwrap();
    let mut batch = WorkerIntentBatch::new("remove-complete-orbit");
    for edge in read
        .relations()
        .iter()
        .filter(|edge| edge.kind.kind_id == KindId(2))
    {
        batch = batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: edge.relation_id,
            },
        )));
    }
    for entity_id in ids {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id },
        )));
    }
    let removed = commit(execution, &runtime, "main", batch);
    let pin = runtime
        .read_truth()
        .pin_read_capability(&removed.snapshot, &[], execution)
        .unwrap();
    let value = pin
        .read_aggregate_lineage(
            execution,
            Scope::new(KindId(2), descriptor().definition, identity),
        )
        .unwrap();
    assert_eq!(value.retired_at(), Some(removed.commit.commit_id));
    assert_eq!(value.latest_identity(), None);
    assert_eq!(value.lineage().count(), 0);
    assert_eq!(value.resolution_steps().count(), 0);
    let missing = AggregateIdentity::from_bytes([255; 32]);
    assert_ne!(identity, missing);
    assert_eq!(
        pin.read_aggregate_lineage(
            execution,
            Scope::new(KindId(2), descriptor().definition, missing)
        )
        .unwrap_err()
        .kind,
        Denial::IdentityUnavailable
    );
    assert_eq!(
        pin.read_aggregate_lineage(
            execution,
            Scope::new(
                KindId(2),
                AggregateDefinitionIdentity::from_bytes([255; 32]),
                identity
            )
        )
        .unwrap_err()
        .kind,
        Denial::DefinitionUnavailable
    );
    drop(value);
    for result in [&born, &removed] {
        release_test_commit_snapshot(&runtime, result);
    }
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn definition_qualification_reads_legal_shared_source_and_split_history_does_not_advance() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let mut other = descriptor();
    other.definition = AggregateDefinitionIdentity::from_bytes([8; 32]);
    other.membership_kind = KindId(6);
    let other_definition = other.definition;
    let registry = crate::schema::aggregate_identity::register_aggregate_membership(
        registry(),
        &SchemaId("test".into()),
        SchemaVersionId(1),
        other,
    )
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build();
    let born = commit(execution, &runtime, "main", birth(&[1, 2, 3, 0]));
    let ids = members(&born, 4);
    let split = commit(
        execution,
        &runtime,
        "main",
        rewire(
            execution,
            &runtime,
            &born.snapshot,
            &[(ids[1], ids[0]), (ids[3], ids[2])],
        ),
    );
    let rows = records(execution, &runtime, &split.snapshot);
    let pin = runtime
        .read_truth()
        .pin_read_capability(&split.snapshot, &[], execution)
        .unwrap();
    let active = rows
        .iter()
        .filter(|(_, row)| row.current_anchor.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        active.len(),
        4,
        "two split groups under each of two definitions"
    );
    assert_eq!(
        active
            .iter()
            .filter(|(_, row)| row.definition == descriptor().definition)
            .count(),
        2
    );
    assert_eq!(
        active
            .iter()
            .filter(|(_, row)| row.definition == other_definition)
            .count(),
        2
    );
    for (_, row) in active {
        let value = pin
            .read_aggregate_lineage(
                execution,
                Scope::new(KindId(2), row.definition, row.identity),
            )
            .unwrap();
        assert_eq!(value.latest_identity(), Some(row.identity));
        assert_eq!(value.resolution_steps().count(), 0);
        assert_eq!(value.lineage().count(), 1);
        assert!(value
            .lineage()
            .all(|event| event.kind() == AggregateLineageKind::SplitFrom));
    }
    for result in [&born, &split] {
        release_test_commit_snapshot(&runtime, result);
    }
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}
