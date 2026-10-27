//! Exact committed source, historical events and retained pure-read custody.
use super::fixtures::*;
use crate::facade::runtime::{
    PinnedAggregateRead, PinnedAggregateReadDenialKind as Denial,
    PinnedAggregateReadScope as Scope, RelationalPinnedRead,
};
use crate::facade::snapshots::SnapshotHandle;
use crate::identity::data::EntityId;
use crate::tests::support::*;
use std::collections::BTreeSet;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    AggregateDefinitionIdentity, AggregateIdentity, AggregateLineageKind as Kind,
};

fn pin(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> RelationalPinnedRead {
    runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .unwrap()
}
fn read(
    pin: &RelationalPinnedRead,
    execution: &ExecutionRequest<'_, '_>,
    member: EntityId,
) -> PinnedAggregateRead {
    pin.read_aggregate(execution, Scope::new(KindId(2), member))
        .unwrap()
}
fn events(
    read: &PinnedAggregateRead,
) -> BTreeSet<(u64, Kind, AggregateIdentity, AggregateIdentity)> {
    read.lineage()
        .map(|event| (event.occurrence().0, event.kind(), event.from(), event.to()))
        .collect()
}

#[test]
fn selected_pins_keep_complete_membership_and_stored_history_after_snapshot_retirement() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 2, 3, 0]));
    let ids = members(&born, 4);
    let original = identity(&records(execution, &runtime, &born.snapshot), ids[0]);
    let old_pin = pin(execution, &runtime, &born.snapshot);
    let original_read = read(&old_pin, execution, ids[2]);
    assert_eq!(original_read.definition(), descriptor().definition);
    assert_eq!(original_read.identity(), original);
    assert_eq!(original_read.members().collect::<Vec<_>>(), ids);
    assert!(events(&original_read).is_empty());
    let split_batch = rewire(
        execution,
        &runtime,
        &born.snapshot,
        &[(ids[1], ids[0]), (ids[3], ids[2])],
    );
    release_test_commit_snapshot(&runtime, &born);
    assert!(runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .is_none());
    let split = commit(execution, &runtime, "main", split_batch);
    let split_identity = identity(&records(execution, &runtime, &split.snapshot), ids[2]);
    let split_pin = pin(execution, &runtime, &split.snapshot);
    let split_read = read(&split_pin, execution, ids[3]);
    let retained_part = read(&split_pin, execution, ids[0]);
    let split_event = (
        split.commit.commit_id.0,
        Kind::SplitFrom,
        split_identity,
        original,
    );
    assert_eq!(split_read.identity(), split_identity);
    assert_eq!(split_read.members().collect::<Vec<_>>(), ids[2..]);
    assert_eq!(retained_part.identity(), original);
    assert_eq!(retained_part.members().collect::<Vec<_>>(), ids[..2]);
    assert_eq!(events(&split_read), BTreeSet::from([split_event]));
    assert_eq!(
        events(&retained_part),
        BTreeSet::from([split_event]),
        "incoming events are actual history too"
    );
    let merged_batch = rewire(
        execution,
        &runtime,
        &split.snapshot,
        &[(ids[1], ids[2]), (ids[3], ids[0])],
    );
    release_test_commit_snapshot(&runtime, &split);
    let merged = commit(execution, &runtime, "main", merged_batch);
    let merged_read = read(
        &pin(execution, &runtime, &merged.snapshot),
        execution,
        ids[1],
    );
    assert_eq!(merged_read.identity(), original);
    assert_eq!(merged_read.members().collect::<Vec<_>>(), ids);
    assert_eq!(
        events(&merged_read),
        BTreeSet::from([
            split_event,
            (
                merged.commit.commit_id.0,
                Kind::MergedInto,
                split_identity,
                original
            )
        ])
    );
    let rows = records(execution, &runtime, &merged.snapshot);
    let retired = rows
        .iter()
        .find(|(_, row)| row.identity == split_identity)
        .unwrap();
    assert_eq!(retired.1.retired, Some(merged.commit.commit_id));
    assert_eq!(
        retired.1.current_anchor, None,
        "historical opposite endpoints are legitimately retired"
    );
    assert_eq!(
        read(&old_pin, execution, ids[0])
            .members()
            .collect::<Vec<_>>(),
        ids
    );
    assert!(events(&read(&old_pin, execution, ids[0])).is_empty());
    assert_eq!(
        events(&read(&split_pin, execution, ids[2])),
        BTreeSet::from([split_event])
    );
    assert_eq!(merged_read.len(), 4);
    assert_eq!(merged_read.anchor(), ids[0]);
    assert!(
        merged_read.reads().entity_records > 0
            && merged_read.reads().relation_records > 0
            && merged_read.reads().aspect_states > 0
    );
    drop((original_read, split_read, retained_part, merged_read));
    release_test_commit_snapshot(&runtime, &merged);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn siblings_share_identity_without_sharing_members_and_pin_schema_ignores_current_configuration() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let mut runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 2, 3, 0]));
    let ids = members(&born, 4);
    create_branch_from_main(&runtime, "pinned-split");
    let selected_main = pin(execution, &runtime, &born.snapshot);
    let forked = commit(
        execution,
        &runtime,
        "pinned-split",
        rewire(
            execution,
            &runtime,
            &born.snapshot,
            &[(ids[1], ids[0]), (ids[3], ids[2])],
        ),
    );
    let selected_fork = pin(execution, &runtime, &forked.snapshot);
    let a = read(&selected_main, execution, ids[0]);
    let b = read(&selected_fork, execution, ids[0]);
    assert_eq!(a.identity(), b.identity());
    assert_eq!(a.members().collect::<Vec<_>>(), ids);
    assert_eq!(b.members().collect::<Vec<_>>(), ids[..2]);
    assert!(events(&a).is_empty());
    assert_eq!(events(&b).len(), 1);
    // Existing Native owner configuration control, not a claim that an empty
    // replacement is a lawful production schema transition.
    runtime.set_schema_registry_for_test(RelationalSchemaRegistry::new());
    assert_eq!(
        read(&selected_main, execution, ids[0])
            .members()
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(
        read(&selected_fork, execution, ids[0])
            .members()
            .collect::<Vec<_>>(),
        ids[..2]
    );
    drop((a, b));
    for result in [&born, &forked] {
        release_test_commit_snapshot(&runtime, result);
    }
    drop(runtime);
    assert_eq!(
        read(&selected_main, execution, ids[1]).identity(),
        read(&selected_fork, execution, ids[1]).identity()
    );
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn observation_pin_and_last_result_clone_keep_only_the_original_read_payer() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = &ExecutionRequest::serial(&setup);
    let runtime = runtime();
    let born = commit(setup_execution, &runtime, "main", birth(&[1, 0]));
    let ids = members(&born, 2);
    let view = runtime
        .read_truth()
        .project_snapshot(&born.snapshot)
        .unwrap();
    let pin = runtime
        .read_truth()
        .pin_view_read_capability(&view, &[], setup_execution)
        .unwrap();
    release_test_commit_snapshot(&runtime, &born);
    drop((view, runtime));
    let memory = SerialMemoryBudget::new(Some(32 * 1024 * 1024));
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let value = pin
        .read_aggregate(
            &ExecutionRequest::serial(&host),
            Scope::new(KindId(2), ids[0]),
        )
        .unwrap();
    let held = memory.charged_memory_bytes();
    assert!(held > 0);
    let clone = value.clone();
    assert_eq!(memory.charged_memory_bytes(), held);
    drop(value);
    assert_eq!(memory.charged_memory_bytes(), held);
    assert_eq!(clone.members().collect::<Vec<_>>(), ids);
    assert!(!clone.is_empty());
    drop(clone);
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn legal_multiple_definitions_refuse_unqualified_source_without_changing_schema_legality() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let mut second = descriptor();
    second.definition = AggregateDefinitionIdentity::from_bytes([8; 32]);
    second.membership_kind = KindId(6);
    let schema = crate::schema::aggregate_identity::register_aggregate_membership(
        registry(),
        &SchemaId("test".into()),
        SchemaVersionId(1),
        second,
    )
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let born = commit(execution, &runtime, "main", birth(&[1, 0]));
    let denial = pin(execution, &runtime, &born.snapshot)
        .read_aggregate(execution, Scope::new(KindId(2), members(&born, 2)[0]))
        .unwrap_err();
    assert_eq!(denial.kind, Denial::AmbiguousAggregateRelation);
    assert_eq!(denial.reads, Default::default());
    assert!(denial.work > 0);
    release_test_commit_snapshot(&runtime, &born);
    let normal = super::fixtures::runtime();
    let born = commit(execution, &normal, "main", birth(&[1, 0]));
    assert_eq!(
        read(
            &pin(execution, &normal, &born.snapshot),
            execution,
            members(&born, 2)[0]
        )
        .len(),
        2
    );
    release_test_commit_snapshot(&normal, &born);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}
