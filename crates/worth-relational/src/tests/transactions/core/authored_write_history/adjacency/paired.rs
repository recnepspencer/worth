//! Complete membership comparison cannot substitute for canonical cancellation.
use super::*;
use crate::facade::runtime::{
    RelationalAdjacencyDirection as ReadDirection, RelationalAdjacencySupportDenial as Denial,
};

#[test]
fn paired_membership_preserves_identity_cardinality_endpoints_and_original_request() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let a = create_entity_outcome(&execution, &runtime, "source");
    let anchor = changed_entities(&a)[0];
    let b = create_entity(&execution, &runtime, "target");
    let c = create_entity(&execution, &runtime, "other-target");
    let empty = runtime.read_truth().project_snapshot(&a.snapshot).unwrap();
    let first = create_relation_outcome(&execution, &runtime, anchor, b, "first");
    let relation = changed_relations(&first)[0];
    let one = runtime
        .read_truth()
        .project_snapshot(&first.snapshot)
        .unwrap();
    assert!(!empty
        .compare_adjacency_support(&execution, &one, anchor, KindId(2), ReadDirection::Outgoing)
        .unwrap());
    // Relation identity is unique within a physical partition. A second
    // partition can genuinely contain a distinct edge with the same endpoints.
    let parallel = create_relation_in_partition(
        &execution,
        &runtime,
        anchor,
        b,
        "parallel",
        crate::identity::data::PartitionId(1),
    );
    let parallel_snapshot = snapshot_for_owner_branch(&runtime, &BranchId("main".into()));
    let two = runtime
        .read_truth()
        .project_snapshot(&parallel_snapshot)
        .unwrap();
    assert!(
        !one.compare_adjacency_support(
            &execution,
            &two,
            anchor,
            KindId(2),
            ReadDirection::Outgoing
        )
        .unwrap(),
        "same endpoints do not collapse parallel relation identity"
    );
    let removed =
        delete_relation_on_branch(&execution, &runtime, parallel, BranchId("main".into()));
    let restored = runtime
        .read_truth()
        .project_snapshot(&removed.snapshot)
        .unwrap();
    assert!(one
        .compare_adjacency_support(
            &execution,
            &restored,
            anchor,
            KindId(2),
            ReadDirection::Outgoing
        )
        .unwrap());
    assert_ne!(
        one.bounded_adjacency_structural_revision(
            &execution,
            anchor,
            KindId(2),
            ReadDirection::Outgoing,
            1
        )
        .unwrap()
        .revision(),
        restored
            .bounded_adjacency_structural_revision(
                &execution,
                anchor,
                KindId(2),
                ReadDirection::Outgoing,
                1
            )
            .unwrap()
            .revision()
    );
    let moved = endpoints(&runtime, execution, relation, anchor, c);
    let moved_view = runtime
        .read_truth()
        .project_snapshot(&moved.snapshot)
        .unwrap();
    assert!(
        !one.compare_adjacency_support(
            &execution,
            &moved_view,
            anchor,
            KindId(2),
            ReadDirection::Outgoing
        )
        .unwrap(),
        "same relation identity still carries both exact endpoints"
    );
    let wrong = EntityId::new(
        anchor.partition_id,
        anchor.local_slot_value(),
        anchor.generation_value() + 1,
    );
    for wrong in [
        wrong,
        EntityId::new(anchor.partition_id, anchor.local_slot_value(), 0),
    ] {
        assert_eq!(
            one.compare_adjacency_support(
                &execution,
                &restored,
                wrong,
                KindId(2),
                ReadDirection::Outgoing
            ),
            Err(Denial::EntityMismatch)
        );
    }
    let canceled = worth_execution::CancellationSource::new();
    canceled.cancel();
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        canceled.token(),
        None,
    );
    assert_eq!(
        one.compare_adjacency_support(
            &ExecutionRequest::serial(&request),
            &restored,
            anchor,
            KindId(2),
            ReadDirection::Outgoing
        ),
        Err(Denial::Execution(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ))
    );
    let no_memory = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        worth_execution::CancellationToken::new(),
        None,
    );
    assert!(matches!(
        one.compare_adjacency_support(
            &ExecutionRequest::serial(&no_memory),
            &restored,
            anchor,
            KindId(2),
            ReadDirection::Outgoing
        ),
        Err(Denial::Execution(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        ))
    ));
    let foreign = runtime_with_test_schema();
    let other = create_entity_outcome(&execution, &foreign, "foreign");
    let foreign_view = foreign
        .read_truth()
        .project_snapshot(&other.snapshot)
        .unwrap();
    assert_eq!(
        one.compare_adjacency_support(
            &execution,
            &foreign_view,
            anchor,
            KindId(2),
            ReadDirection::Outgoing
        ),
        Err(Denial::ForeignOwner)
    );
    release_test_commit_snapshot(&foreign, &other);
    runtime
        .snapshots()
        .release_snapshot(&parallel_snapshot)
        .unwrap();
    for outcome in [&a, &first, &removed, &moved] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn audit_retained_relations_are_not_a_complete_empty_live_list() {
    use crate::config::data::CascadeDeletePolicy;
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit);
    let a = create_entity(&execution, &runtime, "source");
    let b = create_entity(&execution, &runtime, "target");
    let edge = create_relation_outcome(&execution, &runtime, a, b, "audit");
    let old = runtime
        .read_truth()
        .project_snapshot(&edge.snapshot)
        .unwrap();
    let retired = delete_entity(&execution, &runtime, b);
    let current = runtime
        .read_truth()
        .project_snapshot(&retired.snapshot)
        .unwrap();
    assert_eq!(
        old.compare_adjacency_support(&execution, &current, a, KindId(2), ReadDirection::Outgoing),
        Err(Denial::UnsupportedVisibility)
    );
    for outcome in [&edge, &retired] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

mod meaning;
