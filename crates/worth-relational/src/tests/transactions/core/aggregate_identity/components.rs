//! Actual multi-relation Native commits, physical multiplicity and one identity owner.
mod fixtures;
mod proposal;
mod registration;
use fixtures::*;
use worth_foundational::facade::AggregateLineageKind;

#[test]
fn physical_couplings_drive_split_anchor_removal_merge_and_exact_fork_identity() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, birth());
    let ids = ids(&born);
    let original = read(execution, &runtime, &born.snapshot, false, ids[0]);
    let unrelated = read(execution, &runtime, &born.snapshot, false, ids[4]);
    assert_eq!(original.members().collect::<Vec<_>>(), ids[..3]);
    assert_eq!(unrelated.members().collect::<Vec<_>>(), ids[4..]);
    let single_source = read(execution, &runtime, &born.snapshot, true, ids[0]);
    assert_eq!(single_source.members().collect::<Vec<_>>(), ids[..2]);
    assert_ne!(original.identity(), single_source.identity());
    let fork = create_branch_from_main(&runtime, "component-fork");
    let fork_snapshot = snapshot_for_owner_branch(&runtime, &fork);
    let one = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("one-physical-removal").push(delete_edge(&born, 0, 2)),
    );
    assert_eq!(
        read(execution, &runtime, &one.snapshot, false, ids[0]).identity(),
        original.identity()
    );
    assert_eq!(
        read(execution, &runtime, &one.snapshot, false, ids[0])
            .members()
            .collect::<Vec<_>>(),
        ids[..3]
    );
    assert_eq!(
        read(execution, &runtime, &one.snapshot, true, ids[0])
            .members()
            .collect::<Vec<_>>(),
        ids[..1]
    );
    let two = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("inverse-removal").push(delete_edge(&born, 1, 6)),
    );
    assert_eq!(
        read(execution, &runtime, &two.snapshot, false, ids[0])
            .members()
            .collect::<Vec<_>>(),
        ids[..3]
    );
    let split = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("last-physical-removal").push(delete_edge(&born, 5, 6)),
    );
    let fragment = read(execution, &runtime, &split.snapshot, false, ids[1]);
    assert_eq!(fragment.members().collect::<Vec<_>>(), ids[1..3]);
    assert_ne!(fragment.identity(), original.identity());
    assert!(fragment
        .lineage()
        .any(|event| event.kind() == AggregateLineageKind::SplitFrom
            && event.from() == fragment.identity()
            && event.to() == original.identity()
            && event.occurrence() == split.commit.commit_id));
    assert_eq!(
        read(execution, &runtime, &split.snapshot, false, ids[0]).identity(),
        original.identity()
    );
    let removed = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("least-member-removal")
            .push(delete_edge(&born, 2, 6))
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id: ids[1] },
            ))),
    );
    let surviving = read(execution, &runtime, &removed.snapshot, false, ids[2]);
    assert_eq!(surviving.identity(), fragment.identity());
    assert_eq!(surviving.members().collect::<Vec<_>>(), ids[2..3]);
    let other = read(execution, &runtime, &removed.snapshot, false, ids[3]);
    let merged = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("component-merge").push(edge(
            6,
            2,
            EntityReference::Existing(ids[2]),
            EntityReference::Existing(ids[3]),
        )),
    );
    let joined = read(execution, &runtime, &merged.snapshot, false, ids[3]);
    assert_eq!(joined.identity(), fragment.identity());
    assert_eq!(joined.members().collect::<Vec<_>>(), ids[2..4]);
    assert!(joined
        .lineage()
        .any(|event| event.kind() == AggregateLineageKind::MergedInto
            && event.from() == other.identity()
            && event.to() == joined.identity()
            && event.occurrence() == merged.commit.commit_id));
    let selected = runtime
        .read_truth()
        .pin_read_capability(&merged.snapshot, &[], execution)
        .unwrap();
    let resolution = selected
        .read_aggregate_lineage(
            execution,
            crate::facade::runtime::PinnedAggregateLineageScope::for_definition(
                descriptor(false).definition,
                other.identity(),
            ),
        )
        .unwrap();
    assert_eq!(resolution.identity(), other.identity());
    assert_eq!(resolution.retired_at(), Some(merged.commit.commit_id));
    assert_eq!(resolution.latest_identity(), Some(joined.identity()));
    assert_eq!(
        resolution
            .resolution_steps()
            .map(|event| (event.occurrence(), event.kind(), event.from(), event.to()))
            .collect::<Vec<_>>(),
        vec![(
            merged.commit.commit_id,
            AggregateLineageKind::MergedInto,
            other.identity(),
            joined.identity()
        )]
    );
    assert_eq!(
        selected
            .read_aggregate(
                execution,
                crate::facade::runtime::PinnedAggregateReadScope::new(KindId(2), ids[2])
            )
            .unwrap_err()
            .kind,
        crate::facade::runtime::PinnedAggregateReadDenialKind::NotAggregateRelation
    );
    let unchanged = read(execution, &runtime, &merged.snapshot, false, ids[4]);
    assert_eq!(unchanged.identity(), unrelated.identity());
    assert_eq!(unchanged.members().collect::<Vec<_>>(), ids[4..]);
    assert!(unchanged.lineage().next().is_none());
    let forked = read(execution, &runtime, &fork_snapshot, false, ids[2]);
    assert_eq!(forked.identity(), original.identity());
    assert_eq!(forked.members().collect::<Vec<_>>(), ids[..3]);
    assert!(forked.lineage().next().is_none());
}

#[test]
fn checkpoint_and_tail_replay_keep_component_definitions_memberships_and_lineage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = persisted();
    let born = commit(execution, &runtime, birth());
    let ids = ids(&born);
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let split = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("replayed-component-split")
            .push(delete_edge(&born, 0, 2))
            .push(delete_edge(&born, 1, 6))
            .push(delete_edge(&born, 5, 6)),
    );
    let expected = read(execution, &runtime, &split.snapshot, false, ids[1]);
    let (_, recovered) = recover_with(execution, &runtime, persisted);
    let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".into()));
    let actual = read(execution, &recovered, &snapshot, false, ids[1]);
    assert_eq!(actual.identity(), expected.identity());
    assert_eq!(
        actual.members().collect::<Vec<_>>(),
        expected.members().collect::<Vec<_>>()
    );
    let events = |value: &crate::facade::runtime::PinnedAggregateRead| {
        value
            .lineage()
            .map(|event| (event.occurrence(), event.kind(), event.from(), event.to()))
            .collect::<Vec<_>>()
    };
    assert_eq!(events(&actual), events(&expected));
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        recovered.history().commit_envelopes_snapshot()
    );
}

#[test]
fn native_unaccounted_components_keep_original_cancellation_and_no_publication() {
    use worth_execution::{CancellationSource, SerialRequest};
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let cancellation = CancellationSource::new();
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::new(1).unwrap(),
    );
    let host = SerialRequest::from_policy(&policy, cancellation.token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, birth());
    let ids = ids(&born);
    assert!(!execution.has_accounting());
    assert_eq!(
        read(execution, &runtime, &born.snapshot, false, ids[1])
            .members()
            .collect::<Vec<_>>(),
        ids[..3]
    );
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("cancelled-components").push(delete_edge(&born, 1, 6)),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let before = runtime.history().branch_head(&BranchId("main".into()));
    cancellation.cancel();
    assert!(transaction.commit(execution, &runtime).is_err());
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        before
    );
    assert!(runtime.record_identity.pending_snapshot().is_empty());
}
